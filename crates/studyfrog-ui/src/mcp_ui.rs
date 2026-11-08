//! The assistant's handling of what AI clients staged or changed over MCP (STORY-0119): a staged
//! proposal is reviewed in the same card as the assistant's own proposals and applied by the same
//! service; once applied or discarded its file is removed. Pure.

use studyfrog_agent::wiki::Proposal;
use studyfrog_service::agent_vault::Applied;
use studyfrog_service::session::VaultInfo;

use crate::agent_mcp_types::{AgentEffect, McpMsg, McpUi};
use crate::agent_types::{AgentState, Reviewing};
use crate::agent_ui::{fail, notice, with};
use crate::effect::Effect;
use crate::state::State;

fn with_mcp(state: State, f: impl FnOnce(McpUi) -> McpUi) -> (State, Vec<Effect>) {
    with(state, |a| AgentState {
        mcp: f(a.mcp.clone()),
        ..a
    })
}

/// Handle the MCP messages of the assistant.
pub fn update(state: State, msg: McpMsg) -> (State, Vec<Effect>) {
    match msg {
        McpMsg::Loaded(Err(e)) => fail(state, e),
        McpMsg::Loaded(Ok((staged, undo))) => with_mcp(state, |m| McpUi { staged, undo, ..m }),
        McpMsg::Review(id) => review(state, &id),
        McpMsg::Discard(id) => {
            let (state, _) = with_mcp(state, |m| McpUi {
                reviewing: m.reviewing.filter(|r| *r != id),
                ..m
            });
            (state, vec![Effect::Agent(AgentEffect::RemoveStaged(id))])
        }
        McpMsg::Undo(id) => (state, vec![Effect::Agent(AgentEffect::Undo(id))]),
        McpMsg::Undone(Err(e)) => fail(state, e),
        McpMsg::Undone(Ok(info)) => {
            let state = State {
                vault: Some(*info),
                ..state
            };
            let (state, _) = notice(state, "The change was undone.".to_owned());
            (state, vec![Effect::Agent(AgentEffect::LoadMcp)])
        }
    }
}

/// Show a staged proposal in the review card, every change accepted.
fn review(state: State, id: &str) -> (State, Vec<Effect>) {
    let Some(staged) = state.agent.mcp.staged.iter().find(|s| s.proposal.id == id) else {
        return fail(state, "That proposal is gone.".to_owned());
    };
    let p = &staged.proposal;
    let proposal = Proposal {
        summary: format!("{} (from the AI tool {})", p.summary, p.tool),
        changes: p.changes.clone(),
        rejected: staged.rejected.clone(),
    };
    let accepted = vec![true; proposal.changes.len()];
    let id = id.to_owned();
    with(state, |a| AgentState {
        reviewing: Some(Reviewing { proposal, accepted }),
        applied: None,
        error: None,
        tab: crate::agent_types::AgentTab::Wiki,
        mcp: McpUi {
            reviewing: Some(id),
            ..a.mcp
        },
        ..a
    })
}

/// A proposal was applied: say so, refresh, and remove it if it was a staged one.
pub fn applied(state: State, (applied, info): (Applied, VaultInfo)) -> (State, Vec<Effect>) {
    let text = format!(
        "Applied: {} created, {} updated, {} failed.",
        applied.created,
        applied.updated,
        applied.failed.len()
    );
    let staged = state.agent.mcp.reviewing.clone();
    let (state, _) = with(state, |a| AgentState {
        busy: false,
        applied: Some(applied),
        reviewing: None,
        mcp: McpUi {
            reviewing: None,
            ..a.mcp
        },
        ..a
    });
    let state = State {
        vault: Some(info),
        ..state
    };
    let (state, mut effects) = notice(state, text);
    effects.push(Effect::Help(crate::help_types::HelpEffect::Load));
    effects.extend(staged.map(|id| Effect::Agent(AgentEffect::RemoveStaged(id))));
    (state, effects)
}
