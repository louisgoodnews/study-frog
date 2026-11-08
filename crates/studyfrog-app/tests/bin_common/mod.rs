#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers for tests of the real `studyfrog` binary: scratch folders, a small vault, waiting for a
//! file, and a headless instance serving its socket in an isolated home.

pub use std::io::{BufRead, BufReader, Write};
pub use std::path::{Path, PathBuf};
pub use std::process::{Child, Command, Stdio};
pub use std::time::Duration;

pub const BIN: &str = env!("CARGO_BIN_EXE_studyfrog");

/// An empty scratch folder for this test process.
pub fn dir(label: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("sf-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

/// A vault with the stack `fr` and one card.
pub fn vault(root: &Path) {
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

pub fn json(line: &str) -> serde_json::Value {
    serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}"))
}

/// Wait a moment without `thread::sleep` (forbidden by clippy).
pub fn pause(ms: u64) {
    let (_tx, rx) = std::sync::mpsc::channel::<()>();
    let _ = rx.recv_timeout(Duration::from_millis(ms));
}

/// Wait up to `secs` seconds for `path` to exist.
pub fn wait_for_secs(path: &Path, secs: u64) {
    for _ in 0..secs * 20 {
        if path.exists() {
            return;
        }
        pause(50);
    }
    panic!("{} did not appear", path.display());
}

/// A headless instance on `vault` in `home`, serving `run/control.sock`; standard input piped.
pub fn headless(home: &Path, vault: &Path, extra: &[&str]) -> Child {
    let child = Command::new(BIN)
        .args(["--control-stdio", "--vault"])
        .arg(vault)
        .args(extra)
        .env("STUDYFROG_HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    wait_for_secs(&home.join("run/control.json"), 20);
    child
}

/// `studyfrog ctl ARGS` in `home`: exit code and standard output.
pub fn ctl(home: &Path, args: &[&str]) -> (Option<i32>, String) {
    let o = Command::new(BIN)
        .arg("ctl")
        .args(args)
        .env("STUDYFROG_HOME", home)
        .output()
        .unwrap();
    (o.status.code(), String::from_utf8(o.stdout).unwrap())
}
