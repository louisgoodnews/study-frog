#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the simple chat interface in Assistant.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_agent::provider::{Protocol, Provider};
use studyfrog_service::agent::Connections;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn chat_model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Agent;
    let provider = Provider {
        name: "LocalLLM".into(),
        protocol: Protocol::Ollama,
        base_url: "http://127.0.0.1:11434".into(),
        model: "llama3".into(),
        allowed: true,
    };
    let c = Connections {
        providers: vec![provider],
        ..Connections::default()
    };
    apply(&mut m, Msg::Agent(AgentMsg::Loaded(c)));
    apply(&mut m, Msg::Agent(AgentMsg::Select("LocalLLM".into())));
    apply(&mut m, Msg::Agent(AgentMsg::Tab(AgentTab::Chat)));
    m
}

#[test]
fn assistant_chat_interface_renders_and_handles_chat_flow() {
    let m = chat_model();
    let mut h = harness(m);
    h.run_steps(4);

    assert!(
        h.query_by_label_contains("Chatting with LocalLLM")
            .is_some()
    );
    assert!(h.query_by_label("Send").is_some());

    // Type a message
    apply(
        &mut h.state_mut().0,
        Msg::Agent(AgentMsg::ChatInput("Explain spaced repetition".into())),
    );
    h.run_steps(2);

    // Send it
    let fx = apply(&mut h.state_mut().0, Msg::Agent(AgentMsg::ChatSend));
    h.run_steps(2);

    let asked = fx.iter().any(|e| {
        matches!(
            e,
            Effect::AgentChat {
                messages,
                ..
            } if messages.last().map(|m| m.content.as_str()) == Some("Explain spaced repetition")
        )
    });
    assert!(asked, "ChatSend emits AgentChat effect");
    assert!(h.query_by_label("Explain spaced repetition").is_some());

    // Receive assistant reply
    apply(
        &mut h.state_mut().0,
        Msg::Agent(AgentMsg::ChatReplied(Ok(
            "Spaced repetition spaces reviews over increasing intervals.".into(),
        ))),
    );
    h.run_steps(3);

    assert!(h.query_by_label_contains("increasing intervals").is_some());

    // Clear chat
    apply(&mut h.state_mut().0, Msg::Agent(AgentMsg::ChatClear));
    h.run_steps(2);
    assert_eq!(h.state().0.state.agent.chat_messages.len(), 0);

    assert_all_named(&h, "assistant chat interface");
}
