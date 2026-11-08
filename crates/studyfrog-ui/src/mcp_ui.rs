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

fn undone(state: State, info: VaultInfo) -> (State, Vec<Effect>) {
    let state = State {
        vault: Some(info),
        ..state
    };
    let (state, _) = notice(state, "The change was undone.".to_owned());
    (state, vec![Effect::Agent(AgentEffect::LoadMcp)])
}

/// Handle the MCP messages of the assistant. The state is edited in place: it is large, and copies
/// of it fill the stack.
pub fn update(mut state: State, msg: McpMsg) -> (State, Vec<Effect>) {
    if crate::mcp_client_ui::handles(&msg) {
        return crate::mcp_client_ui::update(state, msg);
    }
    let m = &mut state.agent.mcp;
    let effect = match msg {
        McpMsg::Loaded(Err(e)) | McpMsg::Undone(Err(e)) => return fail(state, e),
        McpMsg::Loaded(Ok((staged, undo))) => {
            (m.staged, m.undo) = (staged, undo);
            None
        }
        McpMsg::Review(id) => return review(state, &id),
        McpMsg::Discard(id) => {
            m.reviewing = m.reviewing.take().filter(|r| *r != id);
            Some(AgentEffect::RemoveStaged(id))
        }
        McpMsg::Undo(id) => Some(AgentEffect::Undo(id)),
        McpMsg::Undone(Ok(info)) => return undone(state, *info),
        _ => None,
    };
    (state, effect.map(Effect::Agent).into_iter().collect())
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
