#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The real binary's `--mcp --mcp-http 127.0.0.1:0`: the token it prints (or the one in
//! `secrets.toml`) opens the vault's MCP server on the local listener; nothing else does.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_studyfrog");

fn dir(label: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("sf-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// Start the listener on a free port; the child, the port and the token it printed (if any).
fn start(home: &Path, vault: &Path) -> (Child, u16, Option<String>) {
    let mut child = Command::new(BIN)
        .args(["--mcp", "--mcp-http", "127.0.0.1:0", "--vault"])
        .arg(vault)
        .env("STUDYFROG_HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut err = BufReader::new(child.stderr.take().unwrap());
    let (mut token, mut port) = (None, 0);
    while port == 0 {
        let mut line = String::new();
        assert!(err.read_line(&mut line).unwrap() > 0, "the server stopped");
        if let Some(t) = line
            .trim()
            .strip_prefix("studyfrog --mcp-http: bearer token for this run: ")
        {
            token = Some(t.to_owned());
        }
        if let Some(rest) = line
            .trim()
            .strip_prefix("studyfrog --mcp-http: serving http://127.0.0.1:")
        {
            port = rest.trim_end_matches("/mcp").parse().unwrap();
        }
    }
    (child, port, token)
}

fn post(port: u16, token: &str, body: &str) -> (String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let req = format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    let mut got = String::new();
    s.read_to_string(&mut got).unwrap();
    let (head, body) = got.split_once("\r\n\r\n").unwrap();
    (head.lines().next().unwrap().to_owned(), body.to_owned())
}

const LIST: &str = r#"{"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"studyfrog://vault/summary"}}"#;

#[test]
fn the_printed_token_opens_the_vault_and_a_wrong_one_does_not() {
    let (home, vault) = (dir("mcp-http-home"), dir("mcp-http-vault"));
    std::fs::create_dir_all(vault.join("fr")).unwrap();
    std::fs::write(
        vault.join("fr/_stack.md"),
        "---\nid: 01J00000000000000000000001\nkind: stack\ntitle: French\n---\n",
    )
    .unwrap();
    let (mut child, port, token) = start(&home, &vault);
    let token = token.expect("an ephemeral token is printed");
    let (status, body) = post(port, &token, LIST);
    assert_eq!(status, "HTTP/1.1 200 OK");
    assert!(body.contains("French"), "{body}");
    let (status, _) = post(port, "wrong", LIST);
    assert_eq!(status, "HTTP/1.1 401 Unauthorized");
    let mut out = String::new();
    child.kill().unwrap();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    assert!(out.is_empty(), "nothing is written to standard output");
    child.wait().unwrap();

    std::fs::create_dir_all(home.join("config")).unwrap();
    std::fs::write(
        home.join("config/secrets.toml"),
        "[keys]\nmcp_http = \"fixed-token-123\"\n",
    )
    .unwrap();
    let (mut child, port, printed) = start(&home, &vault);
    assert_eq!(printed, None, "a saved token is never printed");
    assert_eq!(post(port, "fixed-token-123", LIST).0, "HTTP/1.1 200 OK");
    child.kill().unwrap();
    child.wait().unwrap();
    let refused = Command::new(BIN)
        .args(["--mcp", "--mcp-http", "0.0.0.0:0", "--vault"])
        .arg(&vault)
        .env("STUDYFROG_HOME", &home)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(1));
    assert!(
        String::from_utf8(refused.stderr)
            .unwrap()
            .contains("127.0.0.1 only")
    );
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&vault);
}
