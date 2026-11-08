#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Lifecycle events (STORY-0130): raised once by the matching change in a headless engine against a
//! real vault, and streamed to a connection that asked for them with `subscribe`.

mod common;
mod control_common;

use control_common::*;
use studyfrog_ui::control_events::{bus_new, subscribe};

const A: &str = control_common::CARD_A;

fn kinds(rx: &std::sync::mpsc::Receiver<Value>) -> Vec<String> {
    rx.try_iter()
        .map(|v| v["kind"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn each_change_raises_its_event_once() {
    let (vault, logs) = (sample_vault("ev-engine"), temp_dir("ev-engine-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let bus = bus_new();
    e.events = Some(bus.clone());
    let rx = subscribe(&bus);
    ask(
        &mut e,
        "item create card \"Le chat\" --text cat --answer chat --stack french",
    );
    let got: Vec<Value> = rx.try_iter().collect();
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0]["kind"], "item_created");
    assert_eq!(got[0]["data"]["title"], "Le chat");
    assert!(got[0]["data"]["uid"].as_str().unwrap().len() == 26);
    assert!(got[0]["at_ms"].as_i64().unwrap() > 1_800_000_000_000);
    ask(&mut e, &format!("item edit {A} back la réponse"));
    let saved: Vec<Value> = rx.try_iter().collect();
    assert_eq!(saved.len(), 1, "{saved:?}");
    assert_eq!(
        (&saved[0]["kind"], &saved[0]["data"]["uid"]),
        (&json!("item_saved"), &json!(A))
    );
    ask(&mut e, "nav home");
    assert!(kinds(&rx).is_empty(), "navigating raises nothing");
    ask(&mut e, "vault close");
    assert_eq!(kinds(&rx), ["vault_closed"]);
    ask(&mut e, &format!("vault open {}", vault.path.display()));
    assert_eq!(kinds(&rx), ["vault_opened"]);
}

#[test]
fn a_finished_rehearsal_is_reported_once_and_a_stopped_one_not_at_all() {
    let (vault, logs) = (sample_vault("ev-run"), temp_dir("ev-run-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let bus = bus_new();
    e.events = Some(bus.clone());
    let rx = subscribe(&bus);
    for line in [
        "rehearse setup --preset all --stack french",
        "rehearse start",
    ] {
        assert_eq!(ask(&mut e, line)["ok"], true);
    }
    for _ in 0..10 {
        let s = ask(&mut e, "state");
        if s["result"]["rehearse"]["run"]["state"] == "finished" {
            break;
        }
        ask(&mut e, "rehearse flip");
        ask(&mut e, "rehearse rate good");
    }
    let got: Vec<Value> = rx.try_iter().collect();
    let finished: Vec<&Value> = got
        .iter()
        .filter(|v| v["kind"] == "rehearsal_finished")
        .collect();
    assert_eq!(finished.len(), 1, "{got:?}");
    assert_eq!(finished[0]["data"]["mode"], "flip");
    assert!(finished[0]["data"]["items"].as_u64().unwrap() >= 2);
    ask(&mut e, "rehearse leave");
    ask(&mut e, "rehearse setup --preset all --stack french");
    ask(&mut e, "rehearse start");
    ask(&mut e, "rehearse stop");
    assert!(
        !kinds(&rx).contains(&"rehearsal_finished".to_owned()),
        "a stopped run is not finished"
    );
}

#[test]
fn a_subscribed_connection_receives_matching_events_until_it_closes() {
    let (vault, logs) = (sample_vault("ev-conn"), temp_dir("ev-conn-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let bus = bus_new();
    e.events = Some(bus.clone());
    let (inbox, jobs) = channel();
    let mut ctx = conn_ctx(inbox, &logs.path, || {});
    ctx.events = Some(bus.clone());
    let written = Arc::new(Mutex::new(Vec::new()));
    let out: SharedOut = Arc::new(Mutex::new(Box::new(Buf(written.clone()))));
    let input = "subscribe item_created vault_closed\nevents nonsense\n\
                 item create note Hello --stack french\nvault close\n";
    let reader = std::thread::spawn(move || {
        serve_connection(&ctx, Cursor::new(input), &out);
        ctx.inbox.send(Job::Quit).unwrap();
    });
    engine_loop(&mut e, &jobs);
    reader.join().unwrap();
    let mut got = Vec::new();
    for _ in 0..100 {
        got = lines(&String::from_utf8(written.lock().unwrap().clone()).unwrap());
        if got.iter().filter(|v| v["event"] == "lifecycle").count() == 2 {
            break;
        }
        let _ = channel::<()>()
            .1
            .recv_timeout(std::time::Duration::from_millis(20));
    }
    let answers: Vec<&Value> = got.iter().filter(|v| v.get("id").is_some()).collect();
    assert_eq!(answers.len(), 4, "{got:#?}");
    assert_eq!(answers[0]["result"]["following"], true);
    assert_eq!(
        answers[0]["result"]["subscribed"],
        json!(["item_created", "vault_closed"])
    );
    assert!(
        answers[1]["error"]
            .as_str()
            .unwrap()
            .contains("unknown event nonsense")
    );
    let events: Vec<&Value> = got.iter().filter(|v| v["event"] == "lifecycle").collect();
    let names: Vec<&str> = events
        .iter()
        .map(|v| v["data"]["kind"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["item_created", "vault_closed"], "{got:#?}");
    assert_eq!(events[0]["data"]["data"]["title"], "Hello");
}
