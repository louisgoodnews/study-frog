#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The transport against a tiny local server.

use std::io::{Read, Write};
use std::net::TcpListener;

use studyfrog_agent::provider::HttpRequest;
use studyfrog_net::send;

fn serve(status: &str, body: &'static str) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let status = status.to_owned();
    let handle = std::thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        let mut seen = Vec::new();
        let mut buf = [0_u8; 4096];
        loop {
            let n = conn.read(&mut buf).unwrap();
            seen.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&seen).into_owned();
            if let Some((head, rest)) = text.split_once("\r\n\r\n") {
                let want: usize = head
                    .lines()
                    .find_map(|l| {
                        l.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse().unwrap())
                    })
                    .unwrap_or(0);
                if rest.len() >= want {
                    break;
                }
            }
        }
        let reply = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        conn.write_all(reply.as_bytes()).unwrap();
        String::from_utf8_lossy(&seen).into_owned()
    });
    (format!("http://{addr}/api/chat"), handle)
}

fn request(url: &str) -> HttpRequest {
    HttpRequest {
        url: url.into(),
        headers: vec![
            ("Content-Type".into(), "application/json".into()),
            ("x-test".into(), "1".into()),
        ],
        body: "{\"a\":1}".into(),
        timeout_secs: 5,
    }
}

#[test]
fn it_posts_the_body_and_headers_and_returns_status_and_text() {
    let (url, seen) = serve("200 OK", "{\"ok\":true}");
    let r = send(&request(&url)).unwrap();
    assert_eq!((r.status, r.body.as_str()), (200, "{\"ok\":true}"));
    let wire = seen.join().unwrap();
    assert!(wire.starts_with("POST /api/chat"), "{wire}");
    assert!(wire.to_lowercase().contains("x-test: 1") && wire.ends_with("{\"a\":1}"));
}

#[test]
fn an_error_status_is_returned_not_thrown_and_a_dead_server_is_a_sentence() {
    let (url, _seen) = serve("401 Unauthorized", "{\"error\":\"nope\"}");
    assert_eq!(send(&request(&url)).unwrap().status, 401);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let dead = format!("http://{}/x", listener.local_addr().unwrap());
    drop(listener);
    let e = send(&request(&dead)).unwrap_err();
    assert!(e.contains(&dead) && e.contains("connect"), "{e}");
    assert!(!e.contains("x-test"), "headers never appear in errors");
}
