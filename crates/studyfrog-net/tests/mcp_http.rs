#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The local MCP listener: its checks, its request reader and whole exchanges over a real socket
//! on 127.0.0.1 (no other network is touched).

use std::io::{Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};

use studyfrog_agent::mcp_types::{McpPolicy, registry_new};
use studyfrog_net::http_req::read_request;
use studyfrog_net::mcp_http::*;

const TOKEN: &str = "s3cret-token";

#[test]
fn only_the_loopback_address_is_accepted() {
    assert_eq!(loopback_addr("127.0.0.1:8080").unwrap().port(), 8080);
    assert!(loopback_addr("127.0.0.1:0").is_ok());
    for bad in [
        "0.0.0.0:8080",
        "192.168.1.2:80",
        "[::1]:80",
        "localhost:80",
        "8080",
    ] {
        assert!(loopback_addr(bad).is_err(), "{bad}");
    }
}

#[test]
fn host_and_origin_checks() {
    assert!(host_allowed("127.0.0.1:4100", 4100));
    assert!(host_allowed("localhost:4100", 4100));
    assert!(!host_allowed("127.0.0.1:4101", 4100));
    assert!(
        !host_allowed("evil.example:4100", 4100),
        "a rebound name is refused"
    );
    assert!(!host_allowed("127.0.0.1", 4100));
    assert!(origin_allowed("http://localhost:3000"));
    assert!(origin_allowed("https://127.0.0.1"));
    assert!(origin_allowed("http://[::1]:5173"));
    assert!(!origin_allowed("https://evil.example"));
    assert!(!origin_allowed("http://localhost.evil.example"));
    assert!(!origin_allowed("null"));
}

#[test]
fn tokens_are_compared_whole_and_made_fresh() {
    assert!(token_matches(TOKEN, TOKEN));
    assert!(!token_matches("s3cret-tokeN", TOKEN));
    assert!(!token_matches("s3cret", TOKEN));
    assert!(!token_matches("", ""), "an empty token never matches");
    let t = ephemeral_token().unwrap();
    assert_eq!(t.len(), 64);
    assert_ne!(t, ephemeral_token().unwrap());
}

#[test]
fn the_request_reader_keeps_to_its_limits() {
    let raw = "POST /mcp?x=1 HTTP/1.1\r\nHost: a\r\nContent-Length: 2\r\n\r\n{}";
    let req = read_request(&mut Cursor::new(raw)).unwrap();
    assert_eq!(
        (req.method.as_str(), req.path.as_str(), req.body.as_slice()),
        ("POST", "/mcp", b"{}".as_slice())
    );
    let status = |raw: String| read_request(&mut Cursor::new(raw)).unwrap_err().status;
    assert_eq!(
        status("POST /mcp HTTP/1.1\r\nContent-Length: 99999999\r\n\r\n".into()),
        413
    );
    assert_eq!(
        status("POST /mcp HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n".into()),
        501
    );
    assert_eq!(
        status(format!("GET /{} HTTP/1.1\r\n\r\n", "a".repeat(9000))),
        431
    );
    assert_eq!(
        status("POST /mcp HTTP/1.1\r\nContent-Length: 5\r\n\r\n{}".into()),
        400
    );
    assert_eq!(status("hello\r\n\r\n".into()), 400);
}

/// Send raw requests to a listener serving one connection each; the raw answers.
fn exchange(requests: Vec<String>) -> Vec<String> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let n = requests.len();
    let server = std::thread::spawn(move || {
        let guard = HttpGuard {
            token: TOKEN.into(),
            port,
        };
        let reg = registry_new::<()>("studyfrog", "test");
        serve(
            &listener,
            &guard,
            &reg,
            &McpPolicy::default(),
            &mut (),
            Some(n),
        );
    });
    let answers = requests
        .into_iter()
        .map(|r| {
            let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
            s.write_all(r.replace("PORT", &port.to_string()).as_bytes())
                .unwrap();
            let mut got = String::new();
            s.read_to_string(&mut got).unwrap();
            got
        })
        .collect();
    server.join().unwrap();
    answers
}

fn post(headers: &str, body: &str) -> String {
    format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:PORT\r\n{headers}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn whole_exchanges_over_a_local_socket() {
    let auth = format!("Authorization: Bearer {TOKEN}\r\n");
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#;
    let answers = exchange(vec![
        post(&auth, init),
        post(
            &auth,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        ),
        post("", init),
        post("Authorization: Bearer wrong\r\n", init),
        post(&format!("{auth}Origin: https://evil.example\r\n"), init),
        post(&auth, init).replace("Host: 127.0.0.1:PORT", "Host: attacker.example:PORT"),
        format!("GET /mcp HTTP/1.1\r\nHost: 127.0.0.1:PORT\r\n{auth}\r\n"),
        post(&auth, init).replace("POST /mcp", "POST /other"),
        post(
            &format!("{auth}Origin: http://localhost:3000\r\n"),
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        ),
    ]);
    let status = |i: usize| answers[i].lines().next().unwrap().to_owned();
    assert_eq!(status(0), "HTTP/1.1 200 OK");
    let body = answers[0].split("\r\n\r\n").nth(1).unwrap();
    let v: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(v["result"]["serverInfo"]["name"], "studyfrog");
    assert!(answers[0].contains("Content-Type: application/json"));
    assert_eq!(status(1), "HTTP/1.1 202 Accepted");
    assert_eq!(status(2), "HTTP/1.1 401 Unauthorized");
    assert!(answers[2].contains("WWW-Authenticate: Bearer"));
    assert_eq!(status(3), "HTTP/1.1 401 Unauthorized");
    assert_eq!(status(4), "HTTP/1.1 403 Forbidden");
    assert_eq!(status(5), "HTTP/1.1 403 Forbidden");
    assert_eq!(status(6), "HTTP/1.1 405 Method Not Allowed");
    assert!(answers[6].contains("Allow: POST"));
    assert_eq!(status(7), "HTTP/1.1 404 Not Found");
    assert_eq!(
        status(8),
        "HTTP/1.1 200 OK",
        "a page on this machine may call it"
    );
}
