#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `send` hands back the answer's headers (an MCP server names its session in one), against a local
//! one-shot server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;

use studyfrog_agent::provider::HttpRequest;

#[test]
fn the_answer_headers_come_back_in_lower_case() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                length = v.trim().parse().unwrap();
            }
            if line == "\r\n" {
                break;
            }
        }
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        let reply = "HTTP/1.1 200 OK\r\nMcp-Session-Id: abc\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}";
        stream.write_all(reply.as_bytes()).unwrap();
    });
    let request = HttpRequest {
        url: format!("http://127.0.0.1:{port}/mcp"),
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: "{}".into(),
        timeout_secs: 5,
    };
    let response = studyfrog_net::send(&request).unwrap();
    server.join().unwrap();
    assert_eq!((response.status, response.body.as_str()), (200, "{}"));
    assert!(
        response
            .headers
            .contains(&("mcp-session-id".to_owned(), "abc".to_owned()))
    );
}
