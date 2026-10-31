#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The assistant screen: update logic, views, and a whole wiki session and exam through the worker
//! with a fake model.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_agent::provider::{Protocol, Provider};
use studyfrog_service::agent::Connections;
use studyfrog_ui::agent_types::{AgentMsg, FormEdit};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn local() -> Provider {
    Provider {
        name: "Local".into(),
        protocol: Protocol::Ollama,
        base_url: "http://localhost:11434".into(),
        model: "llama".into(),
        allowed: false,
    }
}

fn cloud(allowed: bool) -> Provider {
    Provider {
        name: "Cloud".into(),
        protocol: Protocol::OpenAiCompatible,
        base_url: "https://api.example.com".into(),
        model: "m".into(),
        allowed,
    }
}

fn model(providers: Vec<Provider>) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Agent;
    let c = Connections {
        providers,
        ..Connections::default()
    };
    apply(&mut m, Msg::Agent(AgentMsg::Loaded(c)));
    m
}

fn agent(m: &mut Model, msg: AgentMsg) -> Vec<Effect> {
    apply(m, Msg::Agent(msg))
}

#[test]
fn the_screen_lists_connections_with_consent_and_key_state() {
    let mut m = model(vec![local(), cloud(false)]);
    m.state.agent.connections.with_key.insert("Cloud".into());
    let mut h = harness(m);
    h.run_steps(2);
    for text in [
        "Assistant",
        "Use Local",
        "Use Cloud",
        "runs on this computer",
        "API key saved",
        "Allow sending text from my vault to api.example.com",
        "Remove Cloud",
        "New connection",
        "Add connection",
        "API key (not needed for a local model)",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    h.get_by_label("Allow sending text from my vault to api.example.com")
        .click();
    h.run_steps(2);
    assert!(
        h.state()
            .1
            .contains(&Effect::AllowConnection("Cloud".into(), true))
    );
    assert_all_named(&h, "assistant connections");
}

#[test]
fn adding_a_connection_is_checked_and_a_protocol_fills_in_its_usual_address() {
    let mut m = model(vec![]);
    agent(
        &mut m,
        AgentMsg::Form(FormEdit::Protocol(Protocol::Anthropic)),
    );
    assert_eq!(m.state.agent.form.base_url, "https://api.anthropic.com");
    assert!(agent(&mut m, AgentMsg::AddConnection).is_empty());
    assert!(m.state.agent.error.is_some(), "no name or model yet");
    agent(&mut m, AgentMsg::Form(FormEdit::Name("Claude".into())));
    agent(&mut m, AgentMsg::Form(FormEdit::Model("claude".into())));
    assert!(agent(&mut m, AgentMsg::AddConnection).is_empty());
    assert!(m.state.agent.error.as_ref().unwrap().contains("API key"));
    agent(&mut m, AgentMsg::Form(FormEdit::Key(" sk-1 ".into())));
    let fx = agent(&mut m, AgentMsg::AddConnection);
    let [Effect::SaveConnection { provider, key }] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(
        (provider.name.as_str(), provider.allowed, key.as_deref()),
        ("Claude", false, Some("sk-1"))
    );
}

#[test]
fn a_wiki_request_needs_a_connection_stacks_and_permission_and_a_remote_one_asks_before_sending() {
    let mut m = model(vec![cloud(false)]);
    assert!(agent(&mut m, AgentMsg::PrepareWiki).is_empty());
    assert!(
        m.state
            .agent
            .error
            .as_ref()
            .unwrap()
            .contains("Tick the stacks")
    );
    agent(&mut m, AgentMsg::Folder("french".into(), true));
    assert!(agent(&mut m, AgentMsg::PrepareWiki).is_empty());
    assert!(
        m.state
            .agent
            .error
            .as_ref()
            .unwrap()
            .contains("Allow sending")
    );
    m.state.agent.connections.providers = vec![cloud(true)];
    let fx = agent(&mut m, AgentMsg::PrepareWiki);
    assert_eq!(
        fx,
        [Effect::PrepareWiki {
            folders: vec!["french".into()],
            pdf: None
        }]
    );
    let ctx = studyfrog_service::agent_vault::WikiContext {
        stacks: vec![("french".into(), "French".into())],
        chars: 40,
        ..Default::default()
    };
    assert!(
        agent(&mut m, AgentMsg::WikiPrepared(Ok(Box::new(ctx)))).is_empty(),
        "a remote model waits for the go-ahead"
    );
    assert!(m.state.agent.pending.is_some());
    let fx = agent(&mut m, AgentMsg::SendWiki);
    assert!(matches!(&fx[..], [Effect::AgentChat { .. }]));
    assert!(m.state.agent.busy && m.state.agent.pending.is_none());
}

#[test]
fn a_local_model_is_asked_without_a_second_click_and_proposals_are_reviewed_then_applied() {
    let mut m = model(vec![local()]);
    agent(&mut m, AgentMsg::Folder("french".into(), true));
    let ctx = studyfrog_service::agent_vault::WikiContext {
        stacks: vec![("french".into(), "French".into())],
        ..Default::default()
    };
    let fx = agent(&mut m, AgentMsg::WikiPrepared(Ok(Box::new(ctx))));
    assert!(matches!(&fx[..], [Effect::AgentChat { .. }]), "{fx:?}");
    let proposal = studyfrog_agent::wiki::Proposal {
        summary: "Index".into(),
        changes: vec![
            studyfrog_agent::wiki::Change::CreateNote {
                stack: "french".into(),
                title: "Index".into(),
                body: "- x".into(),
            },
            studyfrog_agent::wiki::Change::CreateCard {
                stack: "french".into(),
                front: "f".into(),
                back: "b".into(),
                source: None,
            },
        ],
        rejected: vec!["Change 3: nope".into()],
    };
    agent(&mut m, AgentMsg::WikiReplied(Ok(Box::new(proposal))));
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Work the vault").click();
    h.run_steps(2);
    for text in [
        "Proposal",
        "Index",
        "Change 3: nope",
        "Accept change 1: New note \"Index\" in french",
        "Apply accepted changes",
        "Discard proposal",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    h.get_by_label("Accept change 2: New card in french: f")
        .click();
    h.run_steps(2);
    h.get_by_label("Apply accepted changes").click();
    h.run_steps(2);
    let fx = h.state().1.clone();
    let applied = fx.iter().find_map(|e| match e {
        Effect::ApplyChanges(c) => Some(c.len()),
        _ => None,
    });
    assert_eq!(
        applied,
        Some(1),
        "only the accepted change goes ahead: {fx:?}"
    );
    assert_all_named(&h, "assistant proposal");
}
