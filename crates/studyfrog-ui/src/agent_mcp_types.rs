//! The assistant's Model Context Protocol parts as plain data: the proposals AI clients staged and
//! the changes they made that can be undone (STORY-0119), and the external tool servers the
//! assistant itself may use with the calls waiting for the user's go-ahead (STORY-0121).

use studyfrog_agent::mcp_client::{Discovered, McpServer};
use studyfrog_agent::mcp_client_chat::ToolCall;
use studyfrog_agent::provider::{Message, Provider};
use studyfrog_service::mcp_staging::Staged;
use studyfrog_service::session::VaultInfo;

use crate::agent_types::ChatPurpose;

/// A change an AI client made that the user can take back: id, what was done, when (Unix ms).
pub type UndoRow = (String, String, i64);

/// A conversation to continue once a tool call is done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatCont {
    /// The model.
    pub provider: Provider,
    /// Its instructions (with the tools).
    pub system: String,
    /// The conversation, the model's tool call last.
    pub messages: Vec<Message>,
    /// What the final answer is for.
    pub purpose: ChatPurpose,
}

/// A tool call the model asked for, with the conversation to continue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCall {
    /// The call.
    pub call: ToolCall,
    /// The conversation.
    pub cont: ChatCont,
}

/// The form for a new tool server: a name and a program to start or an address.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerForm {
    /// Name.
    pub name: String,
    /// `program --args` or `http(s)://...`.
    pub line: String,
}

/// What the assistant shows of MCP.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct McpUi {
    /// Proposals of AI clients waiting for the user.
    pub staged: Vec<Staged>,
    /// The staged proposal under review, if the review card shows one.
    pub reviewing: Option<String>,
    /// Structural changes of AI clients that can be undone.
    pub undo: Vec<UndoRow>,
    /// The external tool servers.
    pub servers: Vec<McpServer>,
    /// What each server offered when it was last asked.
    pub found: Vec<Discovered>,
    /// The new-server form.
    pub form: ServerForm,
    /// A tool call waiting for the user's go-ahead.
    pub approval: Option<PendingCall>,
    /// Tool calls in the answer being worked on.
    pub rounds: usize,
}

/// Messages of the MCP parts of the assistant.
#[derive(Debug, Clone, PartialEq)]
pub enum McpMsg {
    /// The staged proposals and undo entries were read.
    Loaded(Result<(Vec<Staged>, Vec<UndoRow>), String>),
    /// Review a staged proposal (by id) in the review card.
    Review(String),
    /// Throw a staged proposal away.
    Discard(String),
    /// Take back a change of an AI client.
    Undo(String),
    /// The change was taken back; carries the refreshed vault.
    Undone(Result<Box<VaultInfo>, String>),
    /// The tool servers were read or saved.
    Servers(Result<Vec<McpServer>, String>),
    /// The name in the new-server form changed.
    FormName(String),
    /// The program or address in the new-server form changed.
    FormLine(String),
    /// Save the new server.
    AddServer,
    /// Remove a server.
    RemoveServer(String),
    /// Switch a server on or off.
    Enable(String, bool),
    /// Run a server's read-only tools without asking, or not.
    Trust(String, bool),
    /// Allow sending text to a remote server, or not.
    Allow(String, bool),
    /// Ask a server what it offers.
    Discover(String),
    /// What it offers.
    Discovered(Result<Discovered, String>),
    /// The model asked for a tool call.
    ToolRequested(Box<PendingCall>),
    /// The user allows the waiting call.
    Approve,
    /// The user refuses the waiting call.
    Deny,
    /// A tool call is done: the conversation, the call and its text (and whether it is an error).
    ToolDone(Box<PendingCall>, Result<(String, bool), String>),
}

/// Effects of the assistant besides chatting with a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEffect {
    /// Remove a connection and its key.
    RemoveConnection(String),
    /// Allow or forbid sending vault text to a connection.
    AllowConnection(String, bool),
    /// Read the staged proposals and the undo entries of the open vault.
    LoadMcp,
    /// Remove a staged proposal (applied or discarded), then read them again.
    RemoveStaged(String),
    /// Take back a change of an AI client.
    Undo(String),
    /// Read the tool servers.
    LoadServers,
    /// Store the tool servers.
    SaveServers(Vec<McpServer>),
    /// Ask a server what it offers.
    Discover(McpServer),
    /// Run a tool call on its server.
    CallTool(McpServer, Box<PendingCall>),
}
