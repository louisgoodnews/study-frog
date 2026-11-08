//! The assistant's Model Context Protocol parts as plain data (STORY-0119): the proposals AI
//! clients staged and the changes they made that can be undone, their messages and effects.

use studyfrog_service::mcp_staging::Staged;
use studyfrog_service::session::VaultInfo;

/// A change an AI client made that the user can take back: id, what was done, when (Unix ms).
pub type UndoRow = (String, String, i64);

/// What the assistant shows of MCP.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct McpUi {
    /// Proposals of AI clients waiting for the user.
    pub staged: Vec<Staged>,
    /// The staged proposal under review, if the review card shows one.
    pub reviewing: Option<String>,
    /// Structural changes of AI clients that can be undone.
    pub undo: Vec<UndoRow>,
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
}
