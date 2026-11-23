#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The local-model setup panel on the Connections tab (STORY-0206): the audit and status shown,
//! the check and the test buttons asking the worker, and the local model saved after the test.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;

use studyfrog_agent::provider::Protocol;
use studyfrog_service::agent::Connections;
use studyfrog_service::ollama::audit_types::{Audit, CpuInfo, Os};
use studyfrog_service::ollama::catalog::builtin_catalog;
use studyfrog_service::ollama::client::{Installed, OllamaState, status_of};
use studyfrog_service::ollama::fit::rank;
use studyfrog_service::ollama::guide::steps;
use studyfrog_ui::agent_mcp_types::AgentEffect;
use studyfrog_ui::agent_types::AgentMsg;
use studyfrog_ui::agent_types::ollama::{LOCAL_NAME, OllamaMsg, Snapshot};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

const GB: u64 = 1_000_000_000;

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Agent;
    apply(&mut m, Msg::Agent(AgentMsg::Loaded(Connections::default())));
    m
}

/// A snapshot as a check would make it, for a computer with 16 GB and 8 GB free, no graphics card
/// that reports memory, and a service in the given state with the given models.
fn snapshot(installed: Vec<Installed>, version: Option<&str>, binary: bool) -> Snapshot {
    let audit = Audit {
        os: Os::Linux,
        cpu: CpuInfo {
            model: Some("Test CPU".into()),
            logical_cores: Some(8),
            avx2: Some(true),
            ..CpuInfo::default()
        },
        ram_total: Some(16 * GB),
        ram_free: Some(8 * GB),
        gpus: Vec::new(),
        models_dir: None,
        disk_free: None,
    };
    let status = status_of(
        "http://127.0.0.1:11434",
        version.map(str::to_owned),
        version.map(|_| Ok(installed.clone())),
        binary,
    );
    let catalog = builtin_catalog().expect("catalog");
    let suggestions = rank(&catalog, &audit, &status.installed);
    let guide = steps(audit.os, &status, "qwen2.5:1.5b");
    Snapshot {
        audit,
        status,
        suggestions,
        guide,
        catalog_error: None,
    }
}

fn load(m: &mut Model, snap: Snapshot) {
    apply(
        m,
        Msg::Agent(AgentMsg::Ollama(OllamaMsg::Loaded(Box::new(snap)))),
    );
}

#[test]
fn a_missing_ollama_is_said_and_the_setup_starts_at_install() {
    let mut m = model();
    load(&mut m, snapshot(Vec::new(), None, false));
    assert_eq!(
        m.state.agent.ollama.snapshot.as_ref().unwrap().status.state,
        OllamaState::Missing
    );
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Ollama is not installed")
            .is_some()
    );
    assert!(h.query_by_label_contains("1. Install Ollama").is_some());
    assert!(
        h.query_by_label_contains("Graphics card: none could be read")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Disk for models: unknown")
            .is_some()
    );
    assert_all_named(&h, "the setup panel with Ollama missing");
}

#[test]
fn checking_asks_the_worker_for_a_fresh_audit_and_status() {
    let mut h = harness(model());
    h.run_steps(4);
    h.get_by_label("Check this computer").click();
    h.run_steps(2);
    let asked = h
        .state()
        .1
        .iter()
        .any(|e| matches!(e, Effect::Agent(AgentEffect::RefreshOllama)));
    assert!(asked, "the check goes to the worker: {:?}", h.state().1);
}

#[test]
fn a_running_service_with_models_lists_them_and_offers_the_test_button() {
    let installed = vec![Installed {
        name: "qwen2.5:1.5b".into(),
        size_bytes: Some(986_000_000),
        parameter_size: None,
        quantization: None,
    }];
    let mut m = model();
    load(&mut m, snapshot(installed, Some("0.5.4"), true));
    let mut h = harness(m);
    // Tall enough that the suggestions are on screen to be clicked.
    h.set_size(eframe::egui::vec2(1000.0, 6000.0));
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Ollama 0.5.4 is ready with 1 model(s)")
            .is_some()
    );
    assert!(h.query_by_label_contains("Installed in Ollama").is_some());
    h.get_by_label("Test and use llama3.2:1b").click();
    h.run_steps(2);
    let asked =
        h.state().1.iter().any(
            |e| matches!(e, Effect::Agent(AgentEffect::TestLocalModel(n)) if n == "llama3.2:1b"),
        );
    assert!(asked, "the test goes to the worker: {:?}", h.state().1);
    assert_all_named(&h, "the setup panel with models");
}

#[test]
fn a_passed_test_saves_the_model_as_the_local_one() {
    let mut m = model();
    load(&mut m, snapshot(Vec::new(), Some("0.5.4"), true));
    apply(
        &mut m,
        Msg::Agent(AgentMsg::Ollama(OllamaMsg::Choose("llama3.2:1b".into()))),
    );
    assert_eq!(m.state.agent.ollama.testing.as_deref(), Some("llama3.2:1b"));
    let effects = apply(
        &mut m,
        Msg::Agent(AgentMsg::Ollama(OllamaMsg::Tested(
            "llama3.2:1b".into(),
            Ok("OK".into()),
        ))),
    );
    assert!(
        effects.iter().any(
            |e| matches!(e, Effect::Agent(AgentEffect::SetLocalModel(n)) if n == "llama3.2:1b")
        )
    );
    let mut saved = Connections::default();
    saved.providers.push(studyfrog_agent::provider::Provider {
        name: LOCAL_NAME.into(),
        protocol: Protocol::Ollama,
        base_url: "http://127.0.0.1:11434".into(),
        model: "llama3.2:1b".into(),
        allowed: true,
    });
    apply(
        &mut m,
        Msg::Agent(AgentMsg::Ollama(OllamaMsg::LocalSet(Ok(saved)))),
    );
    assert_eq!(m.state.agent.selected.as_deref(), Some(LOCAL_NAME));
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Test passed for llama3.2:1b")
            .is_some()
    );
}

#[test]
fn a_failed_test_saves_nothing() {
    let mut m = model();
    load(&mut m, snapshot(Vec::new(), Some("0.5.4"), true));
    apply(
        &mut m,
        Msg::Agent(AgentMsg::Ollama(OllamaMsg::Choose("llama3.2:1b".into()))),
    );
    let effects = apply(
        &mut m,
        Msg::Agent(AgentMsg::Ollama(OllamaMsg::Tested(
            "llama3.2:1b".into(),
            Err("model not found".into()),
        ))),
    );
    assert!(effects.is_empty(), "nothing is saved after a failed test");
    assert!(m.state.agent.ollama.testing.is_none());
    assert_ne!(m.state.agent.selected.as_deref(), Some(LOCAL_NAME));
}
