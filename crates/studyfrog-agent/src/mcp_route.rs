//! The router of the Model Context Protocol server: one JSON-RPC 2.0 message in, at most one out. Pure
//! and without I/O; the transport (standard input and output, say) lives with the application.

use serde_json::{Map, Value, json};

use crate::mcp_types::{
    INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, McpContent, McpError,
    McpPolicy, McpRegistry, PARSE_ERROR, PROTOCOL_VERSION, mcp_error, tool_permitted,
};
use crate::mcp_uri::{is_fixed, match_template};

/// MCP: a resource that does not exist.
pub const RESOURCE_NOT_FOUND: i64 = -32002;

/// Answer one message. `None` means nothing is sent back (a notification).
pub fn handle_line<C>(
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
    line: &str,
) -> Option<String> {
    let reply = match serde_json::from_str::<Value>(line) {
        Ok(message) => handle_message(reg, policy, ctx, &message),
        Err(_) => Some(failure(
            &Value::Null,
            &mcp_error(PARSE_ERROR, "the message is not JSON"),
        )),
    };
    reply.map(|v| v.to_string())
}

/// Answer one parsed message.
pub fn handle_message<C>(
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
    message: &Value,
) -> Option<Value> {
    let id = message.get("id").cloned();
    let method = message.get("method").and_then(Value::as_str);
    let valid = message.get("jsonrpc").and_then(Value::as_str) == Some("2.0");
    let (true, Some(method)) = (valid, method) else {
        let why = mcp_error(INVALID_REQUEST, "not a JSON-RPC 2.0 request");
        return Some(failure(&id.unwrap_or(Value::Null), &why));
    };
    let params = message.get("params").unwrap_or(&Value::Null);
    let outcome = dispatch(reg, policy, ctx, method, params);
    let id = id?;
    Some(match outcome {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(e) => failure(&id, &e),
    })
}

fn failure(id: &Value, e: &McpError) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": e.code, "message": e.message}})
}

fn dispatch<C>(
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
    method: &str,
    params: &Value,
) -> Result<Value, McpError> {
    match method {
        "initialize" => Ok(initialize(reg)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list(reg, policy)),
        "tools/call" => tools_call(reg, policy, ctx, params),
        "resources/list" => Ok(resources_list(reg, true)),
        "resources/templates/list" => Ok(resources_list(reg, false)),
        "resources/read" => resources_read(reg, ctx, params),
        "prompts/list" => Ok(prompts_list(reg)),
        "prompts/get" => prompts_get(reg, ctx, params),
        m if m.starts_with("notifications/") => Ok(Value::Null),
        m => Err(mcp_error(METHOD_NOT_FOUND, format!("no method {m}"))),
    }
}

fn initialize<C>(reg: &McpRegistry<C>) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {"listChanged": false},
            "resources": {"subscribe": false, "listChanged": false},
            "prompts": {"listChanged": false},
        },
        "serverInfo": {"name": reg.name, "version": reg.version},
    })
}

fn tools_list<C>(reg: &McpRegistry<C>, policy: &McpPolicy) -> Value {
    let tools: Vec<Value> = reg
        .tools
        .iter()
        .filter(|t| tool_permitted(policy, t))
        .map(|t| {
            let schema = serde_json::from_str(t.input_schema).unwrap_or(json!({"type": "object"}));
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": schema,
                "annotations": {
                    "title": t.annotations.title,
                    "readOnlyHint": t.annotations.read_only_hint,
                    "destructiveHint": t.annotations.destructive_hint,
                    "idempotentHint": t.annotations.idempotent_hint,
                    "openWorldHint": t.annotations.open_world_hint,
                },
            })
        })
        .collect();
    json!({"tools": tools})
}

fn content_result(c: &McpContent) -> Value {
    json!({"content": [{"type": "text", "text": c.text}], "isError": c.is_error})
}

fn text_error(text: String) -> Value {
    content_result(&McpContent {
        text,
        is_error: true,
    })
}

fn tools_call<C>(
    reg: &McpRegistry<C>,
    policy: &McpPolicy,
    ctx: &mut C,
    params: &Value,
) -> Result<Value, McpError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| mcp_error(INVALID_PARAMS, "a tool call needs a name"))?;
    let tool = reg
        .tools
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| mcp_error(INVALID_PARAMS, format!("no tool {name}")))?;
    if !tool_permitted(policy, tool) {
        return Ok(text_error(format!(
            "{name} is not allowed: the server was started without permission to change the vault"
        )));
    }
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    if !args.is_object() {
        return Err(mcp_error(INVALID_PARAMS, "arguments must be an object"));
    }
    Ok(match (tool.call)(ctx, &args) {
        Ok(content) => content_result(&content),
        Err(e) => text_error(e.message),
    })
}

fn resources_list<C>(reg: &McpRegistry<C>, fixed: bool) -> Value {
    let chosen = reg
        .resources
        .iter()
        .filter(|r| is_fixed(r.uri_template) == fixed);
    if fixed {
        let list: Vec<Value> = chosen
            .map(|r| json!({"uri": r.uri_template, "name": r.name, "description": r.description, "mimeType": r.mime_type}))
            .collect();
        return json!({"resources": list});
    }
    let list: Vec<Value> = chosen
        .map(|r| json!({"uriTemplate": r.uri_template, "name": r.name, "description": r.description, "mimeType": r.mime_type}))
        .collect();
    json!({"resourceTemplates": list})
}

fn resources_read<C>(reg: &McpRegistry<C>, ctx: &mut C, params: &Value) -> Result<Value, McpError> {
    let uri = params
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| mcp_error(INVALID_PARAMS, "reading a resource needs a uri"))?;
    let hit = reg
        .resources
        .iter()
        .find_map(|r| match_template(r.uri_template, uri).map(|parts| (r, parts)));
    let Some((resource, parts)) = hit else {
        return Err(mcp_error(
            RESOURCE_NOT_FOUND,
            format!("no resource at {uri}"),
        ));
    };
    let c = (resource.read)(ctx, uri, &parts)?;
    let contents: Vec<Value> = std::iter::once(&c)
        .chain(&c.more)
        .map(|c| json!({"uri": c.uri, "mimeType": c.mime_type, "text": c.text}))
        .collect();
    Ok(json!({"contents": contents}))
}

fn prompts_list<C>(reg: &McpRegistry<C>) -> Value {
    let list: Vec<Value> = reg
        .prompts
        .iter()
        .map(|p| {
            let arguments: Vec<Value> = p
                .arguments
                .iter()
                .map(|a| json!({"name": a.name, "description": a.description, "required": a.required}))
                .collect();
            json!({"name": p.name, "description": p.description, "arguments": arguments})
        })
        .collect();
    json!({"prompts": list})
}

fn prompts_get<C>(reg: &McpRegistry<C>, ctx: &mut C, params: &Value) -> Result<Value, McpError> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| mcp_error(INVALID_PARAMS, "a prompt request needs a name"))?;
    let prompt = reg
        .prompts
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| mcp_error(INVALID_PARAMS, format!("no prompt {name}")))?;
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    if let Some(missing) = prompt
        .arguments
        .iter()
        .find(|a| a.required && args.get(a.name).is_none())
    {
        return Err(mcp_error(
            INVALID_PARAMS,
            format!("the argument {} is required", missing.name),
        ));
    }
    let messages = (prompt.get)(ctx, &args).map_err(|e| mcp_error(INTERNAL_ERROR, e.message))?;
    let list: Vec<Value> = messages
        .iter()
        .map(|m| json!({"role": m.role, "content": {"type": "text", "text": m.text}}))
        .collect();
    Ok(json!({"description": prompt.description, "messages": list}))
}
