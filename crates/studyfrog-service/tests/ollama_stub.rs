#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Ollama status probe against a stub HTTP server on 127.0.0.1: ready, running with no models,
//! not running, and missing. The stub answers the two routes the probe uses and nothing else.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use studyfrog_agent::provider::HttpResponse;
use studyfrog_service::ollama::client::{OllamaState, probe};

/// A minimal GET for the test: the address is `http://127.0.0.1:PORT/path`.
fn get(url: &str, _secs: u32) -> Result<HttpResponse, String> {
    let rest = url.strip_prefix("http://").ok_or("no scheme")?;
    let (host, path) = rest.split_once('/').ok_or("no path")?;
    let mut s = TcpStream::connect(host).map_err(|e| e.to_string())?;
    // One write: the stub reads the request in one go.
    let request = format!("GET /{path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    s.write_all(request.as_bytes()).map_err(|e| e.to_string())?;
    let mut text = String::new();
    s.read_to_string(&mut text).map_err(|e| e.to_string())?;
    let (head, body) = text.split_once("\r\n\r\n").ok_or("no body")?;
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .ok_or("no status")?;
    Ok(HttpResponse {
        status,
        body: body.to_owned(),
        headers: Vec::new(),
    })
}

/// Serve `requests` connections with the given models list (or none when `tags` is `None`).
fn serve(requests: usize, version: Option<&'static str>, tags: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    thread::spawn(move || {
        for _ in 0..requests {
            let Ok((mut conn, _)) = listener.accept() else {
                return;
            };
            let mut buf = [0u8; 2048];
            let n = conn.read(&mut buf).unwrap_or(0);
            let line = String::from_utf8_lossy(&buf[..n]).into_owned();
            let (code, body) = if line.starts_with("GET /api/version") {
                match version {
                    Some(v) => ("200 OK", format!(r#"{{"version":"{v}"}}"#)),
                    None => ("404 Not Found", "{}".to_owned()),
                }
            } else if line.starts_with("GET /api/tags") {
                ("200 OK", tags.to_owned())
            } else {
                ("404 Not Found", "{}".to_owned())
            };
            let reply = format!(
                "HTTP/1.1 {code}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = conn.write_all(reply.as_bytes());
        }
    });
    format!("http://{addr}")
}

#[test]
fn a_running_service_with_models_is_ready() {
    let tags = r#"{"models":[{"name":"qwen2.5:1.5b","size":986000000,"details":{"parameter_size":"1.5B","quantization_level":"Q4_K_M"}}]}"#;
    let addr = serve(2, Some("0.5.4"), tags);
    let s = probe(get, &addr, true);
    assert_eq!(s.state, OllamaState::Ready, "{s:?}");
    assert_eq!(s.version.as_deref(), Some("0.5.4"));
    assert_eq!(s.installed.len(), 1);
    assert_eq!(s.installed[0].name, "qwen2.5:1.5b");
}

#[test]
fn a_running_service_with_no_models_says_so() {
    let addr = serve(2, Some("0.5.4"), r#"{"models":[]}"#);
    let s = probe(get, &addr, true);
    assert_eq!(s.state, OllamaState::NoModels);
    assert!(s.installed.is_empty());
}

#[test]
fn a_stopped_service_is_not_running_when_the_program_is_installed() {
    // Port 1 has no listener here; a fixed port avoids a stub's port being reused by another test.
    let addr = "http://127.0.0.1:1";
    assert_eq!(probe(get, addr, true).state, OllamaState::NotRunning);
    assert_eq!(probe(get, addr, false).state, OllamaState::Missing);
}

#[test]
fn a_remote_address_is_never_contacted() {
    let s = probe(get, "http://10.255.255.1:11434", true);
    assert_eq!(s.state, OllamaState::NotRunning);
    assert!(s.note.is_some());
}
