#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Python client (`python/studyfrog`, STORY-0131): its unittest suite against the real binary,
//! each test in a temporary application home with a headless instance (started by the client over
//! pipes, or serving its socket).

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_studyfrog");

fn python_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python")
}

fn has_python() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn the_python_client_suite_passes_against_the_binary() {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed, so the Python client was not tested");
        return;
    }
    let dir = python_dir();
    let out = Command::new("python3")
        .args([
            "-B", "-m", "unittest", "discover", "-v", "-s", "tests", "-t", "tests",
        ])
        .current_dir(&dir)
        .env("PYTHONPATH", &dir)
        .env("STUDYFROG_BIN", BIN)
        .env_remove("STUDYFROG_SOCKET")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{text}");
    assert!(!text.contains("skipped"), "no test may be skipped: {text}");
    assert!(text.contains("\nOK"), "{text}");
}
