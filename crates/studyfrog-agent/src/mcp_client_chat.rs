//! External MCP tools in a conversation with any model (STORY-0121): the tools' names, descriptions
//! and argument schemas go into the instructions, the model asks for one with a small JSON object,
//! and the result goes back as the next user turn. Plain text, so it works the same with every
//! provider the assistant speaks to. Pure.

use serde_json::{Value, json};

use crate::mcp_client::{Discovered, McpServer};
use crate::wiki::json_in;

/// Most tool calls in one answer of the assistant before it must answer without tools.
pub const MAX_TOOL_ROUNDS: usize = 4;
/// Longest tool result passed back to the model.
pub const MAX_RESULT_CHARS: usize = 12_000;

/// A tool call the model asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// The server's name.
    pub server: String,
    /// The tool's name.
    pub tool: String,
    /// The arguments.
    pub arguments: Value,
}

/// The tools of the servers that may be used now, for the model's instructions; empty when there
/// are none.
pub fn tools_block(servers: &[McpServer], found: &[Discovered]) -> String {
    let usable = |d: &&Discovered| {
        servers
            .iter()
            .any(|s| s.name == d.server && crate::mcp_client::server_refusal(s).is_none())
    };
    let tools: Vec<Value> = found
        .iter()
        .filter(usable)
        .flat_map(|d| {
            d.tools.iter().map(|t| {
                json!({"server": d.server, "tool": t.name, "description": t.description,
                    "arguments": t.input_schema, "read_only": t.read_only})
            })
        })
        .collect();
    if tools.is_empty() {
        return String::new();
    }
    format!(
        "\n\nYou may use these external tools. To call one, answer with ONLY this JSON object and \
         nothing else: {{\"tool_call\": {{\"server\": \"<server>\", \"tool\": \"<tool>\", \"arguments\": {{...}}}}}}. \
         The result comes back in the next message; then continue your task. Call a tool only when it \
         helps; the user may refuse a call.\nTools: {}",
        Value::Array(tools)
    )
}

/// The tool call in a reply, if the reply is one.
pub fn parse_tool_call(reply: &str) -> Option<ToolCall> {
    let v = json_in(reply)?;
    let call = v.get("tool_call")?;
    let word = |k: &str| {
        call.get(k)
            .and_then(Value::as_str)
            .map(str::trim)
            .map(str::to_owned)
    };
    let arguments = call.get("arguments").cloned().unwrap_or(json!({}));
    arguments.is_object().then_some(())?;
    Some(ToolCall {
        server: word("server")?,
        tool: word("tool")?,
        arguments,
    })
}

/// The user turn that hands a tool's result (or the refusal) back to the model.
pub fn result_message(call: &ToolCall, outcome: &Result<(String, bool), String>) -> String {
    let (status, text) = match outcome {
        Ok((text, false)) => ("result", text.as_str()),
        Ok((text, true)) => ("error", text.as_str()),
        Err(why) => ("not run", why.as_str()),
    };
    let text: String = text.chars().take(MAX_RESULT_CHARS).collect();
    format!(
        "Tool {} on {} ({status}):\n{text}\n\nContinue with your task.",
        call.tool, call.server
    )
}
