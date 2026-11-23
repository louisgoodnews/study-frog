#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The local-model setup through the control protocol (STORY-0206): `audit`, `ollama status`,
//! `ollama suggest` parse, run headless and answer with the shape an agent reads.

use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_cmd::Request;
use studyfrog_ui::control_cmd::ollama::OllamaCmd;
use studyfrog_ui::control_engine::{Engine, handle};
use studyfrog_ui::control_policy::policy_default;
use studyfrog_ui::control_words::command_from_words;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::Model;

fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

fn engine() -> Engine {
    let dir = std::env::temp_dir().join(format!("sf-ollama-control-{}", std::process::id()));
    Engine {
        model: Model::default(),
        worker: worker_new(Some(dir)),
        policy: policy_default(),
        logs: None,
        clock: Box::new(|| (0.0, 0)),
        events: None,
    }
}

#[test]
fn the_setup_words_parse_to_their_commands() {
    assert_eq!(command_from_words(&words("audit")), Ok(Command::Audit));
    assert_eq!(
        command_from_words(&words("ollama status")),
        Ok(Command::Ollama(OllamaCmd::Status))
    );
    assert_eq!(
        command_from_words(&words("ollama")),
        Ok(Command::Ollama(OllamaCmd::Status))
    );
    assert_eq!(
        command_from_words(&words("ollama suggest")),
        Ok(Command::Ollama(OllamaCmd::Suggest))
    );
    assert!(command_from_words(&words("ollama pull qwen")).is_err());
}

#[test]
fn the_audit_and_the_status_answer_with_their_shape() {
    let mut e = engine();
    let audit = handle(
        &mut e,
        &Request {
            id: serde_json::json!(1),
            cmd: Command::Audit,
        },
    );
    let v: serde_json::Value = serde_json::from_str(&audit).expect("json");
    assert_eq!(v["ok"], true, "{audit}");
    assert!(v["result"]["audit"]["cpu"].is_object(), "{audit}");

    let status = handle(
        &mut e,
        &Request {
            id: serde_json::json!(2),
            cmd: Command::Ollama(OllamaCmd::Status),
        },
    );
    let v: serde_json::Value = serde_json::from_str(&status).expect("json");
    let state = v["result"]["ollama"]["state"].as_str().unwrap_or("");
    assert!(
        ["missing", "not_running", "no_models", "ready"].contains(&state),
        "{status}"
    );
    assert!(
        v["result"]["steps"]
            .as_array()
            .is_some_and(|s| s.len() == 4)
    );
}

#[test]
fn the_suggestions_answer_with_a_ranked_list() {
    let mut e = engine();
    let answer = handle(
        &mut e,
        &Request {
            id: serde_json::json!(3),
            cmd: Command::Ollama(OllamaCmd::Suggest),
        },
    );
    let v: serde_json::Value = serde_json::from_str(&answer).expect("json");
    let list = v["result"]["suggestions"].as_array().expect("list");
    assert!(!list.is_empty(), "{answer}");
    assert_eq!(list[0]["rank"], 1);
    for s in list {
        let fits = s["fits"].as_bool().unwrap_or(false);
        let fit = s["fit"].as_str().unwrap_or("");
        assert_eq!(fits, fit == "video_memory" || fit == "memory", "{s}");
    }
}
