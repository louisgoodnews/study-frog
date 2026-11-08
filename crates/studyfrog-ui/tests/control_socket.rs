#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The control protocol over a real Unix socket in a temporary directory: the files under `run/`,
//! a client session, `studyfrog ctl`'s exchange and a followed log.

mod common;
mod control_common;

use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;

use control_common::*;
use studyfrog_ui::control_client::{exchange, parse_ctl_args, request_line};
use studyfrog_ui::control_socket::{bind, publish, serve, socket_path};

/// A short pause for the log follower (a channel that never receives, so no sleeping call).
fn short_wait() {
    let (tx, rx) = channel::<()>();
    let _ = rx.recv_timeout(Duration::from_millis(20));
    drop(tx);
}

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn send_line(stream: &UnixStream, line: &str) -> Value {
    writeln!(&*stream, "{line}").unwrap();
    let mut got = String::new();
    BufReader::new(stream).read_line(&mut got).unwrap();
    serde_json::from_str(&got).unwrap()
}

#[test]
fn the_socket_is_private_published_refuses_a_second_instance_and_is_removed_after() {
    let home = temp_dir("ctl-sock-files");
    let run = home.path.join("run");
    let listener = bind(&run).unwrap();
    assert_eq!((mode(&run), mode(&socket_path(&run))), (0o700, 0o600));
    let guard = publish(&run, "headless", Some("/v"), 5).unwrap();
    let meta: Value = serde_json::from_str(&std::fs::read_to_string(&guard.meta).unwrap()).unwrap();
    assert_eq!(meta["pid"], std::process::id());
    assert_eq!(
        (meta["mode"].clone(), meta["vault"].clone()),
        (json!("headless"), json!("/v"))
    );
    let again = bind(&run).unwrap_err();
    assert_eq!(again.kind(), std::io::ErrorKind::AddrInUse);
    drop(guard);
    assert!(!socket_path(&run).exists() && !run.join("control.json").exists());
    drop(listener);
}

#[test]
fn a_socket_left_by_an_instance_that_is_gone_is_replaced() {
    let home = temp_dir("ctl-sock-stale");
    let run = home.path.join("run");
    std::fs::create_dir_all(&run).unwrap();
    drop(UnixListener::bind(socket_path(&run)).unwrap());
    assert!(
        socket_path(&run).exists(),
        "a stale socket file is left behind"
    );
    assert!(bind(&run).is_ok());
}

#[test]
fn clients_drive_the_instance_and_follow_its_log_over_the_socket() {
    let (vault, home) = (sample_vault("ctl-sock"), temp_dir("ctl-sock-home"));
    let (run, logs) = (home.path.join("run"), home.path.join("logs"));
    let mut e = engine(&vault.path, &logs, policy_default());
    let listener = bind(&run).unwrap();
    let (inbox, jobs) = channel();
    serve(listener, conn_ctx(inbox.clone(), &logs, short_wait));
    let socket = socket_path(&run);
    let client = std::thread::spawn(move || {
        let follower = UnixStream::connect(&socket).unwrap();
        let first = send_line(&follower, "logs --tail 0 --follow --level warn");
        let s = UnixStream::connect(&socket).unwrap();
        let answers = vec![
            first,
            send_line(&s, "nav library"),
            send_line(&s, r#"{"id": 8, "cmd": "query", "args": ["type = card"]}"#),
            send_line(&s, "nav nowhere"),
        ];
        let mut event = String::new();
        BufReader::new(&follower).read_line(&mut event).unwrap();
        let mut out = Vec::new();
        let ok = exchange(
            &socket,
            &request_line(&["stack".into(), "settings".into(), "french".into()]),
            &mut out,
        )
        .unwrap();
        inbox.send(Job::Quit).unwrap();
        (answers, event, ok, String::from_utf8(out).unwrap())
    });
    engine_loop(&mut e, &jobs);
    let (answers, event, ok, printed) = client.join().unwrap();
    assert_eq!(
        answers[0]["result"],
        json!({"entries": [], "following": true})
    );
    assert_eq!(answers[1]["result"]["screen"], "library");
    assert_eq!(
        (
            answers[2]["id"].clone(),
            answers[2]["result"]["count"].clone()
        ),
        (json!(8), json!(2))
    );
    assert_eq!(answers[3]["ok"], false);
    let event: Value = serde_json::from_str(&event).unwrap();
    assert_eq!(event["event"], "log");
    assert_eq!(event["data"]["message"], "unknown screen nowhere");
    assert!(ok, "ctl's exchange reports success");
    assert_eq!(lines(&printed)[0]["result"]["folder"], "french");
}

#[test]
fn ctl_takes_a_socket_and_the_words_of_one_command() {
    let words = |l: &[&str]| l.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let a = parse_ctl_args(&words(&[
        "--socket",
        "/s",
        "item",
        "edit",
        "X",
        "title",
        "Two words",
    ]))
    .unwrap();
    assert_eq!(a.socket.as_deref(), Some(Path::new("/s")));
    let line: Value = serde_json::from_str(&request_line(&a.words)).unwrap();
    assert_eq!(
        line,
        json!({"id": 1, "cmd": "item", "args": ["edit", "X", "title", "Two words"]})
    );
    let parsed = parse_request(&request_line(&a.words)).unwrap();
    assert!(format!("{:?}", parsed.cmd).contains("Two words"));
    assert!(parse_ctl_args(&[]).unwrap_err().contains("give a command"));
    assert!(parse_ctl_args(&words(&["--socket"])).is_err());
    let missing = exchange(
        Path::new("/nonexistent/control.sock"),
        "state",
        &mut Vec::new(),
    );
    assert!(
        missing
            .unwrap_err()
            .to_string()
            .contains("is StudyFrog running?")
    );
}
