//! The assistant's external tool servers (STORY-0121): registering them, switching them, and asking
//! what they offer. Tool calls are in [`crate::mcp_tool_calls`]. Pure: each step takes the state
//! and gives it back changed (edited in place, since the state is large and copies of it fill the
//! stack).

use studyfrog_agent::mcp_client::{McpServer, McpTransport};
use studyfrog_agent::mcp_client_cfg::{server_problem, split_command};

use crate::agent_mcp_types::{AgentEffect, McpMsg};
use crate::agent_ui::fail;
use crate::effect::Effect;
use crate::state::State;

type Step = (State, Vec<Effect>);

fn save(state: State, servers: Vec<McpServer>) -> Step {
    (
        state,
        vec![Effect::Agent(AgentEffect::SaveServers(servers))],
    )
}

/// The servers with one of them changed.
fn changed(state: State, name: &str, f: impl Fn(&mut McpServer)) -> Step {
    let mut list = state.agent.mcp.servers.clone();
    list.iter_mut().filter(|s| s.name == name).for_each(f);
    save(state, list)
}

fn transport_of(line: &str) -> Option<McpTransport> {
    if line.starts_with("http://") || line.starts_with("https://") {
        return Some(McpTransport::Http {
            url: line.to_owned(),
        });
    }
    split_command(line).map(|(command, args)| McpTransport::Stdio { command, args })
}

fn add_server(mut state: State) -> Step {
    let form = std::mem::take(&mut state.agent.mcp.form);
    let Some(transport) = transport_of(form.line.trim()) else {
        state.agent.mcp.form = form;
        return fail(
            state,
            "Give the program to start or the address.".to_owned(),
        );
    };
    let server = McpServer {
        name: form.name.trim().to_owned(),
        transport,
        enabled: true,
        trust_read_only: false,
        allowed: false,
    };
    if let Some(why) = server_problem(&server, &state.agent.mcp.servers) {
        state.agent.mcp.form = form;
        return fail(state, why);
    }
    let mut list = state.agent.mcp.servers.clone();
    list.push(server);
    save(state, list)
}

/// True for the messages about tool servers and tool calls (the others are about staging).
pub fn handles(msg: &McpMsg) -> bool {
    !matches!(
        msg,
        McpMsg::Loaded(_)
            | McpMsg::Review(_)
            | McpMsg::Discard(_)
            | McpMsg::Undo(_)
            | McpMsg::Undone(_)
    )
}

fn discover(mut state: State, name: &str) -> Step {
    let Some(s) = state
        .agent
        .mcp
        .servers
        .iter()
        .find(|s| s.name == name)
        .cloned()
    else {
        return (state, Vec::new());
    };
    state.agent.busy = true;
    state.agent.error = None;
    (state, vec![Effect::Agent(AgentEffect::Discover(s))])
}

fn discovered(mut state: State, d: studyfrog_agent::mcp_client::Discovered) -> Step {
    state.agent.busy = false;
    let found = &mut state.agent.mcp.found;
    found.retain(|f| f.server != d.server);
    found.push(d);
    (state, Vec::new())
}

/// Handle the tool-server messages of the assistant (see [`handles`]).
pub fn update(mut state: State, msg: McpMsg) -> Step {
    match msg {
        McpMsg::Servers(Err(e)) | McpMsg::Discovered(Err(e)) => return fail(state, e),
        McpMsg::Servers(Ok(servers)) => state.agent.mcp.servers = servers,
        McpMsg::FormName(name) => state.agent.mcp.form.name = name,
        McpMsg::FormLine(line) => state.agent.mcp.form.line = line,
        McpMsg::AddServer => return add_server(state),
        McpMsg::RemoveServer(name) => {
            let mut list = state.agent.mcp.servers.clone();
            list.retain(|s| s.name != name);
            return save(state, list);
        }
        McpMsg::Enable(n, on) => return changed(state, &n, |s| s.enabled = on),
        McpMsg::Trust(n, on) => return changed(state, &n, |s| s.trust_read_only = on),
        McpMsg::Allow(n, on) => return changed(state, &n, |s| s.allowed = on),
        McpMsg::Discover(name) => return discover(state, &name),
        McpMsg::Discovered(Ok(d)) => return discovered(state, d),
        other => return crate::mcp_tool_calls::update(state, other),
    }
    (state, Vec::new())
}
