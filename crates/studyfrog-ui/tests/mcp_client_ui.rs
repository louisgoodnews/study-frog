#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The assistant as an MCP client in the application: tool servers registered and discovered on the
//! Connections tab, and tool calls during an exam with a fake model and a fake local tool server
//! (a shell script): changing tools wait for "Allow", trusted read-only tools run at once.

mod common;

use std::sync::Mutex;

use common::{run_to_idle, sample_vault, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_agent::provider::{HttpRequest, HttpResponse, Protocol, Provider};
use studyfrog_ui::agent_mcp_types::McpMsg;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_with_host};
use studyfrog_ui::host::Host;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

static ASKED: Mutex<Vec<String>> = Mutex::new(Vec::new());
static NEXT_TOOL: Mutex<&str> = Mutex::new("save");

/// A model that asks for the tool in `NEXT_TOOL` first and asks a question once it saw a result.
fn fake_model(req: &HttpRequest) -> Result<HttpResponse, String> {
    ASKED.lock().unwrap().push(req.body.clone());
    let last = req.body.rsplit("\"content\":").next().unwrap_or("");
    let reply = if last.contains("on fake (") {
        "What is a cell?".to_owned()
    } else {
        let tool = *NEXT_TOOL.lock().unwrap();
        format!(
            r#"{{"tool_call": {{"server": "fake", "tool": "{tool}", "arguments": {{"q": "cells"}}}}}}"#
        )
    };
    let escaped = reply.replace('\\', "\\\\").replace('"', "\\\"");
    let body = format!("{{\"message\":{{\"role\":\"assistant\",\"content\":\"{escaped}\"}}}}");
    Ok(HttpResponse {
        headers: Vec::new(),
        status: 200,
        body,
    })
}

fn script() -> String {
    format!(
        "{}/../studyfrog-service/tests/fixtures/fake_mcp_server.sh",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn send(model: &mut Model, worker: &mut Worker, msg: Msg) {
    let fx = apply(model, msg);
    run_to_idle(model, worker, fx);
}

fn mcp(m: McpMsg) -> Msg {
    Msg::Agent(AgentMsg::Mcp(m))
}

/// A model and worker with a vault, the fake server registered and discovered, and a local model.
fn ready(label: &str) -> (Model, Worker, common::TempDir) {
    let vault = sample_vault(label);
    let cfg = temp_dir(&format!("{label}-cfg"));
    let host = Host {
        transport: Some(fake_model),
        ..Host::default()
    };
    let mut worker = worker_with_host(Some(cfg.path.clone()), None, host);
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    let open = vec![Effect::OpenVault(vault.path.display().to_string())];
    run_to_idle(&mut model, &mut worker, open);
    let local = Provider {
        name: "Local".into(),
        protocol: Protocol::Ollama,
        base_url: "http://localhost:11434".into(),
        model: "llama".into(),
        allowed: false,
    };
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::SaveConnection {
            provider: local,
            key: None,
        }],
    );
    send(&mut model, &mut worker, Msg::Navigate(Screen::Agent));
    send(
        &mut model,
        &mut worker,
        mcp(McpMsg::FormName("fake".into())),
    );
    send(
        &mut model,
        &mut worker,
        mcp(McpMsg::FormLine(format!("sh \"{}\"", script()))),
    );
    send(&mut model, &mut worker, mcp(McpMsg::AddServer));
    send(
        &mut model,
        &mut worker,
        mcp(McpMsg::Discover("fake".into())),
    );
    (model, worker, vault)
}

#[test]
fn a_tool_server_is_registered_discovered_and_shown_on_the_connections_tab() {
    let (mut model, mut worker, _vault) = ready("mcp-client-tab");
    assert_eq!(model.state.agent.mcp.servers.len(), 1);
    assert_eq!(model.state.agent.mcp.found[0].tools.len(), 2);
    send(&mut model, &mut worker, mcp(McpMsg::FormName("web".into())));
    send(
        &mut model,
        &mut worker,
        mcp(McpMsg::FormLine("https://tools.example.org/mcp".into())),
    );
    send(&mut model, &mut worker, mcp(McpMsg::AddServer));
    let saved =
        std::fs::read_to_string(worker.config_dir.clone().unwrap().join("mcp_servers.toml"))
            .unwrap();
    assert!(saved.contains("name = \"web\"") && saved.contains("trust_read_only = false"));
    send(
        &mut model,
        &mut worker,
        Msg::Agent(AgentMsg::Tab(AgentTab::Connections)),
    );
    let mut h = common::views::harness(model);
    h.run_steps(2);
    for text in [
        "Tool servers (MCP)",
        "Use fake",
        "Run read-only tools of fake without asking",
        "Discover fake",
        "lookup",
        "read-only",
        "changes things",
        "Resource: fake://library",
        "Prompt: cite",
        "Allow sending text to tools.example.org",
        "Add tool server",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    h.get_by_label("Run read-only tools of fake without asking")
        .click_accesskit();
    h.run_steps(2);
    let (mut model, effects) = std::mem::take(h.state_mut());
    run_to_idle(&mut model, &mut worker, effects);
    assert!(
        model.state.agent.mcp.servers[0].trust_read_only,
        "saved and read back"
    );
}

fn last_request() -> String {
    ASKED.lock().unwrap().last().cloned().unwrap()
}

#[test]
fn exam_tool_calls_ask_first_unless_trusted_and_read_only() {
    let (mut model, mut worker, _vault) = ready("mcp-client-exam");
    send(
        &mut model,
        &mut worker,
        Msg::Agent(AgentMsg::Folder("french".into(), true)),
    );
    *NEXT_TOOL.lock().unwrap() = "save";
    send(&mut model, &mut worker, Msg::Agent(AgentMsg::StartExam));
    assert!(
        last_request().contains("tool_call"),
        "the tools are in the instructions"
    );
    let waiting = model
        .state
        .agent
        .mcp
        .approval
        .clone()
        .expect("save waits for the user");
    assert_eq!(waiting.call.tool, "save");
    let mut h = common::views::harness(model);
    h.run_steps(2);
    assert!(
        h.query_all_by_label("Allow Assistant to run tool save on server fake?")
            .count()
            >= 1
    );
    h.get_by_label("Allow this call").click();
    h.run_steps(2);
    let (mut model, effects) = std::mem::take(h.state_mut());
    run_to_idle(&mut model, &mut worker, effects);
    assert!(
        last_request().contains("ran save"),
        "the result went back to the model"
    );
    let chat = &model.state.agent.exam.chat;
    assert_eq!(chat.last().unwrap().content, "What is a cell?");

    send(&mut model, &mut worker, Msg::Agent(AgentMsg::StartExam));
    send(&mut model, &mut worker, mcp(McpMsg::Deny));
    assert!(
        last_request().contains("not run"),
        "a refusal goes back too"
    );
    assert!(model.state.agent.mcp.approval.is_none());

    *NEXT_TOOL.lock().unwrap() = "lookup";
    send(
        &mut model,
        &mut worker,
        mcp(McpMsg::Trust("fake".into(), true)),
    );
    send(&mut model, &mut worker, Msg::Agent(AgentMsg::StartExam));
    assert!(
        model.state.agent.mcp.approval.is_none(),
        "trusted read-only tools do not ask"
    );
    assert!(last_request().contains("ran lookup"));
    assert_eq!(
        model.state.agent.exam.chat.last().unwrap().content,
        "What is a cell?"
    );

    send(&mut model, &mut worker, Msg::Agent(AgentMsg::PrepareWiki));
    assert!(
        last_request().contains("ran lookup"),
        "a wiki task can use the tools as well"
    );
    let why = model.state.agent.error.clone().unwrap_or_default();
    assert!(
        why.contains("JSON proposal"),
        "the fake model never proposes: {why}"
    );
}
