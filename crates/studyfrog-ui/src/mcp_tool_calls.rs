//! Tool calls a model asks for during an assistant conversation (STORY-0121): a call runs at once
//! only when the tool says it only reads and the user trusts its server; every other call waits
//! for "Allow". The result, or why the call was not run, goes back to the model. Pure: each step
//! takes the state and gives it back changed.

use studyfrog_agent::mcp_client::{needs_approval, server_refusal};
use studyfrog_agent::mcp_client_chat::{MAX_TOOL_ROUNDS, result_message, tools_block};
use studyfrog_agent::provider::{Message, Role};

use crate::agent_mcp_types::{AgentEffect, ChatCont, McpMsg, PendingCall};
use crate::effect::Effect;
use crate::state::State;

type Step = (State, Vec<Effect>);

/// The instructions with the usable tools added, and a fresh count of tool calls; used whenever a
/// conversation turn starts from the user.
pub fn with_tools(mut state: State, system: String) -> (State, String) {
    let m = &mut state.agent.mcp;
    let system = system + &tools_block(&m.servers, &m.found);
    m.rounds = 0;
    (state, system)
}

/// Hand a result (or why the call was not run) back to the model.
fn continue_with(
    mut state: State,
    p: PendingCall,
    outcome: &Result<(String, bool), String>,
) -> Step {
    let ChatCont {
        provider,
        system,
        mut messages,
        purpose,
    } = p.cont;
    messages.push(Message {
        role: Role::User,
        content: result_message(&p.call, outcome),
    });
    state.agent.busy = true;
    state.agent.mcp.approval = None;
    state.agent.mcp.rounds += 1;
    (
        state,
        vec![Effect::AgentChat {
            provider,
            system,
            messages,
            purpose,
        }],
    )
}

fn run(mut state: State, p: PendingCall) -> Step {
    let server = state
        .agent
        .mcp
        .servers
        .iter()
        .find(|s| s.name == p.call.server)
        .cloned();
    let Some(s) = server else {
        return continue_with(state, p, &Err("the server was removed".to_owned()));
    };
    state.agent.mcp.approval = None;
    (
        state,
        vec![Effect::Agent(AgentEffect::CallTool(s, Box::new(p)))],
    )
}

/// The model asked for a tool: run it, ask the user, or say why not.
fn requested(mut state: State, p: PendingCall) -> Step {
    let m = &state.agent.mcp;
    let server = m.servers.iter().find(|s| s.name == p.call.server);
    let tool = m
        .found
        .iter()
        .filter(|d| d.server == p.call.server)
        .flat_map(|d| d.tools.iter())
        .find(|t| t.name == p.call.tool);
    let refused = match (server, tool) {
        _ if m.rounds >= MAX_TOOL_ROUNDS => {
            Some("too many tool calls; answer without tools now".to_owned())
        }
        (Some(s), Some(_)) => server_refusal(s),
        _ => Some("there is no such tool".to_owned()),
    };
    let ask = matches!((server, tool), (Some(s), Some(t)) if needs_approval(s, t));
    match (refused, ask) {
        (Some(why), _) => continue_with(state, p, &Err(why)),
        (None, false) => run(state, p),
        (None, true) => {
            state.agent.mcp.approval = Some(p);
            (state, Vec::new())
        }
    }
}

/// Handle `ToolRequested`, `Approve`, `Deny` and `ToolDone`.
pub fn update(state: State, msg: McpMsg) -> Step {
    let waiting = state.agent.mcp.approval.clone();
    match (msg, waiting) {
        (McpMsg::ToolRequested(p), _) => requested(state, *p),
        (McpMsg::ToolDone(p, outcome), _) => continue_with(state, *p, &outcome),
        (McpMsg::Approve, Some(p)) => run(state, p),
        (McpMsg::Deny, Some(p)) => continue_with(
            state,
            p,
            &Err("the user did not allow this call".to_owned()),
        ),
        _ => (state, Vec::new()),
    }
}
