//! The Model Context Protocol server as plain data (STORY-0117): tools, resource templates and
//! prompts are records holding function pointers, and a registry is a record of lists. Adding one is
//! adding an entry; nothing here is a trait object. `C` is whatever the functions need to work with
//! (the open vault, for the application).

use serde_json::Value;

/// The protocol version the server speaks.
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// JSON-RPC: the text was not JSON.
pub const PARSE_ERROR: i64 = -32700;
/// JSON-RPC: the JSON was not a request.
pub const INVALID_REQUEST: i64 = -32600;
/// JSON-RPC: no such method.
pub const METHOD_NOT_FOUND: i64 = -32601;
/// JSON-RPC: the parameters do not fit the method.
pub const INVALID_PARAMS: i64 = -32602;
/// JSON-RPC: the server failed.
pub const INTERNAL_ERROR: i64 = -32603;

/// Why a call failed, as a JSON-RPC error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpError {
    /// The JSON-RPC error code.
    pub code: i64,
    /// What went wrong, in a sentence.
    pub message: String,
}

/// What a tool says about itself, so a client can decide whether to ask before calling it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpToolAnnotations {
    /// A name for people.
    pub title: &'static str,
    /// The tool changes nothing.
    pub read_only_hint: bool,
    /// The tool may delete or overwrite.
    pub destructive_hint: bool,
    /// Calling it again with the same arguments changes nothing more.
    pub idempotent_hint: bool,
    /// The tool reaches beyond the vault (the network, say).
    pub open_world_hint: bool,
}

/// The text a tool returns, and whether it is an error the model should see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpContent {
    /// The answer.
    pub text: String,
    /// The call failed in a way the model can react to.
    pub is_error: bool,
}

/// Runs a tool with its arguments.
pub type McpToolFn<C> = fn(&mut C, &Value) -> Result<McpContent, McpError>;

/// Reads a resource: the address that was asked for and its `{name}` parts as (name, value).
pub type McpReadFn<C> =
    fn(&mut C, &str, &[(String, String)]) -> Result<McpResourceContent, McpError>;

/// Makes the messages of a prompt from its arguments.
pub type McpPromptFn<C> = fn(&mut C, &Value) -> Result<Vec<McpPromptMessage>, McpError>;

/// A function the model can call.
#[derive(Debug, Clone, Copy)]
pub struct McpTool<C> {
    /// Unique name.
    pub name: &'static str,
    /// What it does.
    pub description: &'static str,
    /// The JSON Schema of its arguments, as JSON text.
    pub input_schema: &'static str,
    /// What it says about itself.
    pub annotations: McpToolAnnotations,
    /// Without permission to write, the tool still runs but only stages its change for the user.
    pub staged: bool,
    /// Run it with the arguments.
    pub call: McpToolFn<C>,
}

/// What a resource holds when it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpResourceContent {
    /// The address that was read.
    pub uri: String,
    /// `text/markdown`, `application/json`, ...
    pub mime_type: &'static str,
    /// The text.
    pub text: String,
    /// Further parts of the same resource (the schedule as JSON next to the Markdown, say).
    pub more: Vec<McpResourceContent>,
}

/// Something the model can read, at an address with `{name}` parts such as `studyfrog://items/{id}`;
/// an address without parts is a single fixed resource.
#[derive(Debug, Clone, Copy)]
pub struct McpResourceTemplate<C> {
    /// The address pattern.
    pub uri_template: &'static str,
    /// A name for people.
    pub name: &'static str,
    /// What is behind it.
    pub description: &'static str,
    /// The kind of text it holds.
    pub mime_type: &'static str,
    /// Read it; the parts of the address that matched `{name}` are passed as (name, value).
    pub read: McpReadFn<C>,
}

/// An argument of a prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpPromptArgument {
    /// Its name.
    pub name: &'static str,
    /// What it is for.
    pub description: &'static str,
    /// The prompt cannot be made without it.
    pub required: bool,
}

/// One message of a prompt: `user` or `assistant`, and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPromptMessage {
    /// Who says it.
    pub role: &'static str,
    /// What is said.
    pub text: String,
}

/// A ready-made conversation start a client can offer to its user.
#[derive(Debug, Clone, Copy)]
pub struct McpPrompt<C> {
    /// Unique name.
    pub name: &'static str,
    /// What it is for.
    pub description: &'static str,
    /// What it takes.
    pub arguments: &'static [McpPromptArgument],
    /// Make the messages from the arguments.
    pub get: McpPromptFn<C>,
}

/// Everything a server offers.
#[derive(Debug, Clone)]
pub struct McpRegistry<C> {
    /// The server's name, for `initialize`.
    pub name: &'static str,
    /// The server's version, for `initialize`.
    pub version: &'static str,
    /// The tools.
    pub tools: Vec<McpTool<C>>,
    /// The resources.
    pub resources: Vec<McpResourceTemplate<C>>,
    /// The prompts.
    pub prompts: Vec<McpPrompt<C>>,
}

/// What the server lets through without asking: changes and deletions are off unless allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct McpPolicy {
    /// Tools that change the vault may be called.
    pub allow_write: bool,
    /// Tools that may delete or overwrite may be called (needs `allow_write` too).
    pub allow_destructive: bool,
}

/// An empty registry for a server of this name and version.
pub fn registry_new<C>(name: &'static str, version: &'static str) -> McpRegistry<C> {
    McpRegistry {
        name,
        version,
        tools: Vec::new(),
        resources: Vec::new(),
        prompts: Vec::new(),
    }
}

/// The registry with this tool added (a tool of the same name is replaced).
pub fn register_tool<C>(mut reg: McpRegistry<C>, tool: McpTool<C>) -> McpRegistry<C> {
    reg.tools.retain(|t| t.name != tool.name);
    reg.tools.push(tool);
    reg
}

/// The registry with this resource added (one of the same address pattern is replaced).
pub fn register_resource<C>(
    mut reg: McpRegistry<C>,
    resource: McpResourceTemplate<C>,
) -> McpRegistry<C> {
    reg.resources
        .retain(|r| r.uri_template != resource.uri_template);
    reg.resources.push(resource);
    reg
}

/// The registry with this prompt added (one of the same name is replaced).
pub fn register_prompt<C>(mut reg: McpRegistry<C>, prompt: McpPrompt<C>) -> McpRegistry<C> {
    reg.prompts.retain(|p| p.name != prompt.name);
    reg.prompts.push(prompt);
    reg
}

/// Whether the policy lets a tool with these annotations be listed and called.
pub fn tool_allowed(policy: &McpPolicy, a: &McpToolAnnotations) -> bool {
    a.read_only_hint || (policy.allow_write && (!a.destructive_hint || policy.allow_destructive))
}

/// Whether the policy lets this tool be listed and called: a staging tool always may, since it
/// changes nothing until the user applies what it staged.
pub fn tool_permitted<C>(policy: &McpPolicy, tool: &McpTool<C>) -> bool {
    tool.staged || tool_allowed(policy, &tool.annotations)
}

/// An error with a code and a sentence.
pub fn mcp_error(code: i64, message: impl Into<String>) -> McpError {
    McpError {
        code,
        message: message.into(),
    }
}
