#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The real binary publishes lifecycle events: `studyfrog ctl subscribe` attached to a headless
//! instance's socket streams the item created through standard input (STORY-0130).

mod bin_common;

use bin_common::*;

#[test]
fn ctl_subscribe_streams_the_events_of_a_headless_instance() {
    let (home, v) = (dir("ev-bin-home"), dir("ev-bin-vault"));
    vault(&v);
    let mut server = headless(&home, &v, &[]);
    let mut sub = Command::new(BIN)
        .args(["ctl", "subscribe", "item_created"])
        .env("STUDYFROG_HOME", &home)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(sub.stdout.take().unwrap()).lines();
    let first = json(&lines.next().unwrap().unwrap());
    assert_eq!(
        first["result"]["subscribed"],
        serde_json::json!(["item_created"])
    );
    let mut stdin = server.stdin.take().unwrap();
    writeln!(
        stdin,
        "item create card Hund --text dog --answer Hund --stack fr"
    )
    .unwrap();
    let event = json(&lines.next().unwrap().unwrap());
    assert_eq!(event["event"], "lifecycle");
    assert_eq!(event["data"]["kind"], "item_created");
    assert_eq!(event["data"]["data"]["title"], "Hund");
    let (code, text) = ctl(&home, &["subscribe", "nothing_like_this"]);
    assert_eq!(code, Some(1), "{text}");
    sub.kill().unwrap();
    let _ = sub.wait();
    drop(stdin);
    assert!(server.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}
