#![allow(clippy::unwrap_used)]
//! Tests for the FrogScript runtime evaluation engine.

use studyfrog_script::log_stream::{get_stderr_messages, get_stdout_messages};
use studyfrog_script::run_frogscript;
use studyfrog_script::val::{ScriptItem, Val};

fn sample_items() -> Vec<ScriptItem> {
    vec![
        ScriptItem {
            uid: Some("01JABCDEF".into()),
            title: "Synapse Structure".into(),
            prompt: "What is the synaptic cleft?".into(),
            answer: "The gap between pre- and post-synaptic neurons.".into(),
            kind: "card".into(),
            stack: "Neuroscience/Synapses".into(),
            tags: vec!["cram-session".into(), "neuro".into()],
            priority: "low".into(),
            difficulty: "medium".into(),
            familiarity: 0.45,
            due_day: Some(20_000),
        },
        ScriptItem {
            uid: Some("01JBCDEFG".into()),
            title: "Action Potential".into(),
            prompt: "What ion causes depolarization?".into(),
            answer: "Sodium (Na+) influx.".into(),
            kind: "card".into(),
            stack: "Neuroscience/Synapses".into(),
            tags: vec!["neuro".into()],
            priority: "low".into(),
            difficulty: "low".into(),
            familiarity: 0.85,
            due_day: Some(20_005),
        },
        ScriptItem {
            uid: Some("01JCDEFGH".into()),
            title: "Photosynthesis".into(),
            prompt: "Where does light reaction occur?".into(),
            answer: "Thylakoid membrane.".into(),
            kind: "card".into(),
            stack: "Biology/Plants".into(),
            tags: vec!["bio".into()],
            priority: "low".into(),
            difficulty: "hard".into(),
            familiarity: 0.30,
            due_day: Some(20_001),
        },
    ]
}

#[test]
fn evaluates_user_query_and_filtering_pipeline() {
    let items = sample_items();
    let src = r#"
        cards = from stack "Neuroscience/Synapses" | where tag "cram-session"
        cards | set priority "high" | where familiarity < 0.7
    "#;
    let (res, logs) = run_frogscript(src, items, 1_700_000_000_000).unwrap();

    let Val::List(out) = res else {
        panic!("expected list result");
    };
    assert_eq!(out.len(), 1);

    let Val::Item(it) = &out[0] else {
        panic!("expected item");
    };
    assert_eq!(it.title, "Synapse Structure");
    assert_eq!(it.priority, "high");
    assert!(it.familiarity < 0.7);
    assert_eq!(logs.entries.len(), 0);
}

#[test]
fn evaluates_variable_manipulation_and_semicolon_chaining() {
    let src = "a = 20; b = 30; total = a + b; log_info(total)";
    let (res, logs) = run_frogscript(src, Vec::new(), 1_700_000_000_000).unwrap();

    assert_eq!(res, Val::Void);
    let stdout = get_stdout_messages(&logs);
    assert_eq!(stdout, vec!["50"]);
}

#[test]
fn evaluates_stdout_and_stderr_logging() {
    let src = r#"
        log_info("Starting deck optimization")
        log_warn("Low memory warning")
        log_error("Something went wrong!")
    "#;
    let (_, logs) = run_frogscript(src, Vec::new(), 1_700_000_000_000).unwrap();

    let stdout = get_stdout_messages(&logs);
    let stderr = get_stderr_messages(&logs);

    assert_eq!(
        stdout,
        vec!["Starting deck optimization", "Low memory warning"]
    );
    assert_eq!(stderr, vec!["Something went wrong!"]);
}

#[test]
fn evaluates_closures_over_piped_items() {
    let items = sample_items();
    let src = r#"
        from stack * | where type "card" | {
            card -> if card.difficulty == "low": card | set priority "low"
        }
    "#;
    let (res, _) = run_frogscript(src, items, 1_700_000_000_000).unwrap();

    let Val::List(out) = res else {
        panic!("expected list");
    };
    assert_eq!(out.len(), 3);
}

#[test]
fn evaluates_datetime_and_builtins() {
    let now_ms = 1_728_000_000_000; // Epoch ms
    let src = "t = today(); d = days_ago(5); diff = t - d; print(diff)";

    let (_, logs) = run_frogscript(src, Vec::new(), now_ms).unwrap();
    let stdout = get_stdout_messages(&logs);
    assert_eq!(stdout, vec!["5"]);
}
