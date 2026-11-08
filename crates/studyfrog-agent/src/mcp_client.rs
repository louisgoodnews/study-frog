//! The assistant as an MCP client (STORY-0121): the servers the user registered, the messages a
//! client sends and reads, and when a tool call needs the user's go-ahead. Pure; the service moves
//! the bytes (a child process's standard input and output, or HTTP).

use serde_json::{Value, json};

use crate::mcp_types::PROTOCOL_VERSION;
use crate::provider::host_of;

/// How to reach a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpTransport {
    /// A program on this computer, talked to over its standard input and output.
    Stdio {
        /// The program.
        command: String,
        /// Its arguments.
        args: Vec<String>,
    },
    /// A Streamable HTTP endpoint.
    Http {
        /// Its address.
        url: String,
    },
}

/// An external MCP server the user registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServer {
    /// A unique name, such as `zotero`.
    pub name: String,
    /// How to reach it.
    pub transport: McpTransport,
    /// The assistant may use it.
    pub enabled: bool,
    /// Tools that say they only read run without asking.
    pub trust_read_only: bool,
    /// The user allowed sending text to a remote server (asked only for remote addresses).
    pub allowed: bool,
}

/// A tool a server offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTool {
    /// Its name.
    pub name: String,
    /// What it does.
    pub description: String,
    /// The JSON Schema of its arguments.
    pub input_schema: Value,
    /// It says it changes nothing.
    pub read_only: bool,
    /// It says it may delete or overwrite.
    pub destructive: bool,
}

/// What a server offers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Discovered {
    /// The server's name as registered.
    pub server: String,
    /// What it calls itself, with its version.
    pub info: String,
    /// Its tools.
    pub tools: Vec<RemoteTool>,
    /// Its resources (addresses and templates).
    pub resources: Vec<String>,
    /// Its prompts.
    pub prompts: Vec<String>,
}

/// True when the server runs on this computer, so nothing leaves it.
pub fn server_is_local(s: &McpServer) -> bool {
    match &s.transport {
        McpTransport::Stdio { .. } => true,
        McpTransport::Http { url } => matches!(host_of(url), "localhost" | "127.0.0.1" | "::1"),
    }
}

/// Why the assistant may not talk to this server now, or `None` when it may.
pub fn server_refusal(s: &McpServer) -> Option<String> {
    if !s.enabled {
        return Some(format!("The tool server {} is switched off.", s.name));
    }
    (!server_is_local(s) && !s.allowed).then(|| {
        format!(
            "Text may not be sent to the tool server {} until you allow it.",
            s.name
        )
    })
}

/// True when calling this tool needs the user's go-ahead first: always, unless the tool says it only
/// reads and the user trusts the server's read-only tools.
pub fn needs_approval(s: &McpServer, t: &RemoteTool) -> bool {
    !(t.read_only && s.trust_read_only)
}

/// A request as one line of JSON.
pub fn client_request(id: u64, method: &str, params: &Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

/// A notification as one line of JSON.
pub fn client_notification(method: &str) -> String {
    json!({"jsonrpc": "2.0", "method": method}).to_string()
}

/// What `initialize` sends.
pub fn initialize_params(version: &str) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {},
        "clientInfo": {"name": "studyfrog-assistant", "version": version},
    })
}

/// The answer to request `id` in this message: its result or its error. `None` when the message is
/// something else (a notification, another answer, or not JSON).
pub fn response_for(message: &Value, id: u64) -> Option<Result<Value, String>> {
    if message.get("id").and_then(Value::as_u64) != Some(id) {
        return None;
    }
    if let Some(e) = message.get("error") {
        let text = e.get("message").and_then(Value::as_str).unwrap_or("error");
        return Some(Err(format!("the server answered: {text}")));
    }
    Some(Ok(message.get("result").cloned().unwrap_or(Value::Null)))
}

fn hint(v: &Value, key: &str) -> bool {
    v.get("annotations")
        .and_then(|a| a.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// The tools of a `tools/list` result. A tool without annotations counts as changing things.
pub fn tools_of(result: &Value) -> Vec<RemoteTool> {
    let list = result.get("tools").and_then(Value::as_array);
    list.into_iter()
        .flatten()
        .filter_map(|t| {
            Some(RemoteTool {
                name: t.get("name")?.as_str()?.to_owned(),
                description: t
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned(),
                input_schema: t
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or(json!({"type": "object"})),
                read_only: hint(t, "readOnlyHint"),
                destructive: hint(t, "destructiveHint"),
            })
        })
        .collect()
}

/// The `field` of every entry of the list `key` of a result.
pub fn names_of(result: &Value, key: &str, field: &str) -> Vec<String> {
    let list = result.get(key).and_then(Value::as_array);
    list.into_iter()
        .flatten()
        .filter_map(|r| r.get(field).and_then(Value::as_str).map(str::to_owned))
        .collect()
}

/// `name version` from an `initialize` result.
pub fn info_of(result: &Value) -> String {
    let info = result.get("serverInfo");
    let word = |k: &str| {
        info.and_then(|i| i.get(k))
            .and_then(Value::as_str)
            .unwrap_or("")
    };
    format!("{} {}", word("name"), word("version"))
        .trim()
        .to_owned()
}

/// The text of a `tools/call` result and whether it is an error.
pub fn call_text(result: &Value) -> (String, bool) {
    let parts = result.get("content").and_then(Value::as_array);
    let text: Vec<&str> = parts
        .into_iter()
        .flatten()
        .filter_map(|c| c.get("text").and_then(Value::as_str))
        .collect();
    let error = result
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    (text.join("\n"), error)
}

/// The JSON-RPC messages in an HTTP answer's body: plain JSON, or the `data:` lines of server-sent
/// events.
pub fn http_messages(body: &str) -> Vec<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(body.trim()) {
        return vec![v];
    }
    body.lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .filter_map(|d| serde_json::from_str(d.trim()).ok())
        .collect()
}
