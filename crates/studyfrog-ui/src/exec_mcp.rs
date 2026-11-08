//! Performing the assistant's other effects: connections, and what AI clients staged or changed over
//! MCP (STORY-0119).

use studyfrog_service::mcp_staging::{list_staged, remove_staged};
use studyfrog_service::mcp_undo::{list_undo, undo_recorded};

use crate::agent_mcp_types::{AgentEffect, McpMsg, UndoRow};
use crate::agent_types::AgentMsg;
use crate::exec::Worker;
use crate::msg::Msg;

fn mcp(m: McpMsg) -> Msg {
    Msg::Agent(AgentMsg::Mcp(m))
}

/// The staged proposals and undo entries of the open vault (none without a vault).
fn load(w: &Worker) -> Msg {
    let Some(s) = &w.session else {
        return mcp(McpMsg::Loaded(Ok((Vec::new(), Vec::new()))));
    };
    let undo: Vec<UndoRow> = list_undo(&s.root)
        .into_iter()
        .map(|e| (e.id, e.label, e.at_ms))
        .collect();
    let staged = list_staged(s).map_err(|e| e.to_string());
    mcp(McpMsg::Loaded(staged.map(|st| (st, undo))))
}

/// Perform one of these effects.
pub fn run(w: &mut Worker, effect: AgentEffect, _now_ns: i64) -> Msg {
    match effect {
        AgentEffect::RemoveConnection(name) => crate::exec_agent::remove(w, &name),
        AgentEffect::AllowConnection(name, on) => crate::exec_agent::allow(w, &name, on),
        AgentEffect::LoadMcp => load(w),
        AgentEffect::RemoveStaged(id) => match &w.session {
            Some(s) => match remove_staged(&s.root, &id) {
                Ok(()) => load(w),
                Err(e) => mcp(McpMsg::Loaded(Err(e.to_string()))),
            },
            None => load(w),
        },
        AgentEffect::Undo(id) => mcp(McpMsg::Undone(match &mut w.session {
            Some(s) => undo_recorded(s, &id)
                .map(Box::new)
                .map_err(|e| e.to_string()),
            None => Err("no vault is open".to_owned()),
        })),
    }
}
