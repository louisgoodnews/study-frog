//! Performing the assistant's other effects: connections, and what AI clients staged or changed over
//! MCP (STORY-0119), and the external tool servers the assistant uses (STORY-0121).

use studyfrog_service::mcp_client_io::{call_server_tool, discover_server};
use studyfrog_service::mcp_servers::{load_servers, save_servers};
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

fn servers(w: &Worker, save: Option<&[studyfrog_agent::mcp_client::McpServer]>) -> Msg {
    let Some(dir) = w.config_dir.clone() else {
        return mcp(McpMsg::Servers(Err("there is no config folder".to_owned())));
    };
    let saved = save.map_or(Ok(()), |list| {
        save_servers(&dir, list).map_err(|e| e.to_string())
    });
    mcp(McpMsg::Servers(saved.and_then(|()| load_servers(&dir))))
}

/// "Test connection": one tiny request through the transport (STORY-0083, STORY-0084).
fn test_connection(w: &Worker, p: &studyfrog_agent::provider::Provider, now_ns: i64) -> Msg {
    let result = match (w.transport, w.config_dir.as_deref()) {
        (Some(transport), Some(config_dir)) => {
            let ctx = studyfrog_service::agent::Ask {
                config_dir,
                log_dir: w.state_dir.as_deref(),
                now_ms: now_ns / 1_000_000,
            };
            studyfrog_service::agent::test_connection(transport, p, ctx)
        }
        (None, _) => Err("This build cannot reach the network.".to_owned()),
        (_, None) => Err("there is no config folder".to_owned()),
    };
    let done = crate::agent_conn_test::TestMsg::Done(p.name.clone(), result);
    Msg::Agent(AgentMsg::Test(done))
}

/// Perform one of these effects.
pub fn run(w: &mut Worker, effect: AgentEffect, now_ns: i64) -> Msg {
    match effect {
        AgentEffect::RemoveConnection(name) => crate::exec_agent::remove(w, &name),
        AgentEffect::AllowConnection(name, on) => crate::exec_agent::allow(w, &name, on),
        AgentEffect::TestConnection(p) => test_connection(w, &p, now_ns),
        AgentEffect::LoadMcp => load(w),
        AgentEffect::RemoveStaged(id) => match &w.session {
            Some(s) => match remove_staged(&s.root, &id) {
                Ok(()) => load(w),
                Err(e) => mcp(McpMsg::Loaded(Err(e.to_string()))),
            },
            None => load(w),
        },
        AgentEffect::LoadServers => servers(w, None),
        AgentEffect::SaveServers(list) => servers(w, Some(&list)),
        AgentEffect::Discover(s) => mcp(McpMsg::Discovered(discover_server(&s, w.transport))),
        AgentEffect::CallTool(s, p) => {
            let outcome = call_server_tool(&s, w.transport, &p.call.tool, &p.call.arguments);
            mcp(McpMsg::ToolDone(p, outcome))
        }
        AgentEffect::Undo(id) => mcp(McpMsg::Undone(match &mut w.session {
            Some(s) => undo_recorded(s, &id)
                .map(Box::new)
                .map_err(|e| e.to_string()),
            None => Err("no vault is open".to_owned()),
        })),
    }
}
