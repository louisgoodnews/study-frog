#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The connection form's suggested models.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_agent::provider::Protocol;
use studyfrog_service::agent::Connections;
use studyfrog_ui::agent_types::{AgentMsg, FormEdit};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Agent;
    apply(&mut m, Msg::Agent(AgentMsg::Loaded(Connections::default())));
    m
}

fn agent(m: &mut Model, msg: AgentMsg) -> Vec<Effect> {
    apply(m, Msg::Agent(msg))
}

#[test]
fn a_protocol_suggests_its_first_model_until_the_user_types_their_own() {
    let mut m = model();
    agent(
        &mut m,
        AgentMsg::Form(FormEdit::Protocol(Protocol::DeepSeek)),
    );
    assert_eq!(m.state.agent.form.base_url, "https://api.deepseek.com");
    assert_eq!(m.state.agent.form.model, "deepseek-chat");
    agent(&mut m, AgentMsg::Form(FormEdit::Protocol(Protocol::Gemini)));
    assert_eq!(
        m.state.agent.form.model, "gemini-1.5-flash",
        "an untouched suggestion is replaced"
    );
    agent(
        &mut m,
        AgentMsg::Form(FormEdit::Model("my-tuned-model".into())),
    );
    agent(
        &mut m,
        AgentMsg::Form(FormEdit::Protocol(Protocol::DeepSeek)),
    );
    assert_eq!(
        m.state.agent.form.model, "my-tuned-model",
        "the user's own choice stays"
    );
    agent(&mut m, AgentMsg::Form(FormEdit::Model(String::new())));
    agent(&mut m, AgentMsg::Form(FormEdit::Protocol(Protocol::Ollama)));
    assert_eq!(
        m.state.agent.form.model, "",
        "a protocol without suggestions leaves it empty"
    );
}

#[test]
fn the_form_offers_the_suggested_models_of_the_protocol_as_buttons() {
    let mut m = model();
    agent(&mut m, AgentMsg::Form(FormEdit::Protocol(Protocol::Gemini)));
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Suggested models:").is_some());
    h.get_by_label("gemini-1.5-pro").click();
    h.run_steps(3);
    assert_eq!(h.state().0.state.agent.form.model, "gemini-1.5-pro");
    assert_all_named(&h, "assistant connections form");
}

#[test]
fn the_connections_tab_offers_the_mcp_config_only_with_a_vault() {
    let mut h = harness(model());
    h.run_steps(4);
    assert!(h.query_by_label("Copy MCP server config").is_some());
    assert_all_named(&h, "connections with mcp");
    let mut none = Model::default();
    none.state.screen = Screen::Agent;
    let mut h = harness(none);
    h.run_steps(4);
    assert!(h.query_by_label("Copy MCP server config").is_none());
}
