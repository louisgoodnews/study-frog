#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! What AI clients staged over MCP shows on the assistant's "Work the vault" tab, is reviewed and
//! applied like the assistant's own proposals, and their structural changes can be undone there.

mod common;
mod mcp_common;

use common::{run_to_idle, sample_vault as vault_of};
use egui_kittest::kittest::Queryable;
use mcp_common::*;
use studyfrog_ui::agent_mcp_types::McpMsg;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn opened(path: &Path) -> (Model, Worker) {
    let mut model = Model::default();
    model.state.app.tutorial_offered = true;
    let mut worker = worker_new(None);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(path.display().to_string())],
    );
    (model, worker)
}

fn step(model: &mut Model, worker: &mut Worker, msg: Msg) {
    let fx = apply(model, msg);
    run_to_idle(model, worker, fx);
}

fn mcp(m: McpMsg) -> Msg {
    Msg::Agent(AgentMsg::Mcp(m))
}

#[test]
fn a_staged_proposal_is_reviewed_applied_and_then_gone() {
    let vault = vault_of("mcp-staged-ui");
    let mut c = ctx(&vault.path);
    let items = json!({"summary": "Two words", "items": [
        {"kind": "card", "stack": "French", "front": "chat", "back": "cat"},
        {"kind": "card", "stack": "French", "front": "chien", "back": "dog"},
    ]});
    call(&mut c, McpPolicy::default(), "vault_propose_items", items);
    call(
        &mut c,
        McpPolicy::default(),
        "diary_log_entry",
        json!({"date": "2026-11-07"}),
    );
    drop(c);
    let before = markdown_files(&vault.path).len();
    let (mut model, mut worker) = opened(&vault.path);
    step(&mut model, &mut worker, Msg::Navigate(Screen::Agent));
    assert_eq!(model.state.agent.mcp.staged.len(), 2);
    let id_of = |m: &Model, tool: &str| {
        let s = m.state.agent.mcp.staged.iter();
        s.map(|s| &s.proposal)
            .find(|p| p.tool == tool)
            .unwrap()
            .id
            .clone()
    };
    let (id, diary) = (
        id_of(&model, "vault_propose_items"),
        id_of(&model, "diary_log_entry"),
    );
    step(&mut model, &mut worker, mcp(McpMsg::Review(id.clone())));
    assert_eq!(model.state.agent.tab, AgentTab::Wiki);
    let mut h = common::views::harness(model);
    h.run_steps(2);
    for text in [
        "Proposals from AI tools",
        "Review \"Two words\"",
        "Discard \"Two words\"",
        "Accept change 1: New card in french: chat",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    h.get_by_label("Accept change 2: New card in french: chien")
        .click();
    h.run_steps(2);
    let (mut model, effects) = std::mem::take(h.state_mut());
    run_to_idle(&mut model, &mut worker, effects);
    step(&mut model, &mut worker, Msg::Agent(AgentMsg::ApplyProposal));
    assert_eq!(model.state.agent.applied.as_ref().unwrap().created, 1);
    assert_eq!(
        markdown_files(&vault.path).len(),
        before + 1,
        "only the accepted card"
    );
    let left: Vec<&str> = model
        .state
        .agent
        .mcp
        .staged
        .iter()
        .map(|s| s.proposal.id.as_str())
        .collect();
    assert_eq!(
        left,
        [diary.as_str()],
        "the applied proposal's file is removed"
    );
    step(&mut model, &mut worker, mcp(McpMsg::Discard(diary)));
    assert!(model.state.agent.mcp.staged.is_empty());
    assert!(
        !vault.path.join("diary").exists(),
        "a discarded entry is never written"
    );
}

#[test]
fn a_change_by_an_ai_tool_is_undone_from_the_assistant() {
    let vault = vault_of("mcp-undo-ui");
    let before = markdown_files(&vault.path);
    let mut c = ctx(&vault.path);
    let all = McpPolicy {
        allow_write: true,
        allow_destructive: true,
    };
    call(
        &mut c,
        all,
        "vault_delete_item",
        json!({"item": format_ulid(Uid(11))}),
    );
    drop(c);
    let (mut model, mut worker) = opened(&vault.path);
    step(&mut model, &mut worker, Msg::Navigate(Screen::Agent));
    step(
        &mut model,
        &mut worker,
        Msg::Agent(AgentMsg::Tab(AgentTab::Wiki)),
    );
    let (id, label, _) = model.state.agent.mcp.undo[0].clone();
    let mut h = common::views::harness(model);
    h.run_steps(2);
    assert!(h.query_all_by_label("Changes by AI tools").count() >= 1);
    h.get_by_label(&format!("Undo: {label}")).click();
    h.run_steps(2);
    let (mut model, effects) = std::mem::take(h.state_mut());
    assert!(
        effects.iter().any(|e| matches!(e, Effect::Agent(_))),
        "{id}"
    );
    run_to_idle(&mut model, &mut worker, effects);
    assert!(model.state.agent.mcp.undo.is_empty());
    assert_eq!(markdown_files(&vault.path), before);
}
