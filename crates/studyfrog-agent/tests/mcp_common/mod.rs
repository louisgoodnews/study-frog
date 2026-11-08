#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers for the MCP tests: a small registry of test tools, resources and a prompt.
pub use serde_json::{Value, json};
pub use studyfrog_agent::mcp_route::{RESOURCE_NOT_FOUND, handle_line};
pub use studyfrog_agent::mcp_types::{
    McpContent, McpError, McpPolicy, McpPrompt, McpPromptArgument, McpPromptMessage, McpRegistry,
    McpResourceContent, McpResourceTemplate, McpTool, McpToolAnnotations, mcp_error,
    register_prompt, register_resource, register_tool, registry_new,
};
pub use studyfrog_agent::mcp_uri::match_template;

#[derive(Default)]
pub struct Ctx {
    pub changes: u32,
}

pub const READ: McpToolAnnotations = McpToolAnnotations {
    title: "Read",
    read_only_hint: true,
    destructive_hint: false,
    idempotent_hint: true,
    open_world_hint: false,
};
pub const WRITE: McpToolAnnotations = McpToolAnnotations {
    title: "Write",
    read_only_hint: false,
    destructive_hint: false,
    idempotent_hint: false,
    open_world_hint: false,
};
pub const WIPE: McpToolAnnotations = McpToolAnnotations {
    title: "Wipe",
    read_only_hint: false,
    destructive_hint: true,
    idempotent_hint: false,
    open_world_hint: false,
};

pub fn echo(_: &mut Ctx, args: &Value) -> Result<McpContent, McpError> {
    let text = args
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or("nothing");
    Ok(McpContent {
        text: text.to_owned(),
        is_error: false,
    })
}

pub fn change(c: &mut Ctx, _: &Value) -> Result<McpContent, McpError> {
    c.changes += 1;
    Ok(McpContent {
        text: format!("changes: {}", c.changes),
        is_error: false,
    })
}

pub fn fails(_: &mut Ctx, _: &Value) -> Result<McpContent, McpError> {
    Err(mcp_error(-1, "the vault is closed"))
}

pub fn tool(
    name: &'static str,
    a: McpToolAnnotations,
    call: fn(&mut Ctx, &Value) -> Result<McpContent, McpError>,
) -> McpTool<Ctx> {
    McpTool {
        name,
        description: "a test tool",
        input_schema: r#"{"type":"object","properties":{"text":{"type":"string"}}}"#,
        annotations: a,
        call,
    }
}

pub fn read_item(
    _: &mut Ctx,
    uri: &str,
    parts: &[(String, String)],
) -> Result<McpResourceContent, McpError> {
    let id = parts.first().map_or("none", |(_, v)| v.as_str());
    Ok(McpResourceContent {
        uri: uri.to_owned(),
        mime_type: "text/markdown",
        text: format!("item {id}"),
    })
}

pub fn greet(_: &mut Ctx, args: &Value) -> Result<Vec<McpPromptMessage>, McpError> {
    let who = args.get("name").and_then(Value::as_str).unwrap_or("?");
    Ok(vec![McpPromptMessage {
        role: "user",
        text: format!("Hello {who}"),
    }])
}

pub fn registry() -> McpRegistry<Ctx> {
    let mut reg = registry_new("test-server", "1.2.3");
    reg = register_tool(reg, tool("echo", READ, echo));
    reg = register_tool(reg, tool("change", WRITE, change));
    reg = register_tool(reg, tool("wipe", WIPE, change));
    reg = register_tool(reg, tool("broken", READ, fails));
    reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "test://summary",
            name: "Summary",
            description: "the summary",
            mime_type: "application/json",
            read: read_item,
        },
    );
    reg = register_resource(
        reg,
        McpResourceTemplate {
            uri_template: "test://items/{id}",
            name: "Item",
            description: "an item",
            mime_type: "text/markdown",
            read: read_item,
        },
    );
    register_prompt(
        reg,
        McpPrompt {
            name: "greet",
            description: "greets",
            arguments: &[McpPromptArgument {
                name: "name",
                description: "who",
                required: true,
            }],
            get: greet,
        },
    )
}

/// Send one request and parse the answer.
pub fn ask(policy: McpPolicy, ctx: &mut Ctx, request: &Value) -> Value {
    let line = handle_line(&registry(), &policy, ctx, &request.to_string()).expect("an answer");
    serde_json::from_str(&line).unwrap()
}

pub fn call(method: &str, params: &Value) -> Value {
    ask(
        McpPolicy::default(),
        &mut Ctx::default(),
        &json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}),
    )
}

pub fn names(v: &Value, key: &str) -> Vec<String> {
    v["result"][key]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect()
}
