#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The real `studyfrog` binary: `--control-stdio` over pipes, and `studyfrog ctl` attached to it over
//! `run/control.sock` in an isolated application home.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const BIN: &str = env!("CARGO_BIN_EXE_studyfrog");

fn dir(label: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("sf-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn vault(root: &Path) {
    let write = |rel: &str, text: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    write(
        "fr/_stack.md",
        "---\nid: 01J00000000000000000000001\nkind: stack\ntitle: French\n---\n",
    );
    write(
        "fr/a.md",
        "---\nid: 01J00000000000000000000010\nkind: card\ntitle: Chat\n---\ncat\n???\nchat\n",
    );
}

fn json(line: &str) -> serde_json::Value {
    serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}"))
}

fn wait_for(path: &Path) {
    for _ in 0..100 {
        if path.exists() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let _ = rx.recv_timeout(Duration::from_millis(50));
        drop(tx);
    }
    panic!("{} did not appear", path.display());
}

#[test]
fn control_stdio_answers_every_line_on_stdout_and_nothing_else() {
    let (home, v) = (dir("ctl-bin-home"), dir("ctl-bin-vault"));
    vault(&v);
    let out = Command::new(BIN)
        .args(["--control-stdio", "--no-control-socket", "--vault"])
        .arg(&v)
        .env("STUDYFROG_HOME", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child = out;
    let input = "state\nquery type = card\nrehearse setup --preset all\nrehearse start\nrehearse flip\nnope\n";
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let done = child.wait_with_output().unwrap();
    assert!(done.status.success());
    let stdout = String::from_utf8(done.stdout).unwrap();
    let answers: Vec<_> = stdout.lines().map(json).collect();
    assert_eq!(answers.len(), 6, "{stdout}");
    assert_eq!(
        answers[0]["result"]["vault"]["stacks"][0]["title"],
        "French"
    );
    assert_eq!(answers[1]["result"]["rows"][0]["title"], "Chat");
    assert_eq!(answers[4]["result"]["rehearse"]["run"]["answer"], "chat");
    assert_eq!(answers[5]["ok"], false);
    assert!(String::from_utf8(done.stderr).unwrap().is_empty());
}

#[test]
fn ctl_attaches_to_a_headless_instance_through_its_socket() {
    let (home, v) = (dir("ctl-sock-home"), dir("ctl-sock-vault"));
    vault(&v);
    let mut server = Command::new(BIN)
        .args(["--control-stdio", "--vault"])
        .arg(&v)
        .env("STUDYFROG_HOME", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let socket = home.join("run/control.sock");
    wait_for(&home.join("run/control.json"));
    let ctl = |args: &[&str]| {
        let o = Command::new(BIN)
            .arg("ctl")
            .args(args)
            .env("STUDYFROG_HOME", &home)
            .output()
            .unwrap();
        (o.status.code(), String::from_utf8(o.stdout).unwrap())
    };
    let (code, text) = ctl(&["stack", "open", "fr"]);
    assert_eq!(code, Some(0), "{text}");
    assert_eq!(json(text.trim())["result"]["selection"]["stack"], "fr");
    let (code, text) = ctl(&["item", "delete", "01J00000000000000000000010"]);
    assert_eq!(code, Some(1));
    assert!(
        json(text.trim())["error"]
            .as_str()
            .unwrap()
            .contains("destructive")
    );
    let mut stdin = server.stdin.take().unwrap();
    writeln!(stdin, "state").unwrap();
    let mut first = String::new();
    BufReader::new(server.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    assert_eq!(
        json(&first)["result"]["screen"],
        "library",
        "stdin sees what ctl did"
    );
    drop(stdin);
    assert!(server.wait().unwrap().success());
    assert!(!socket.exists(), "the socket is removed on exit");
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}
