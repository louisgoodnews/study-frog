//! UI types and effect definitions for marking a folder as a stack or normal folder (ADR 0011, TASK-0217).

use studyfrog_service::bulk::Snapshot;

/// Effect to execute stack mark or unmark actions.
#[derive(Debug, Clone, PartialEq)]
pub enum StackMarkEffect {
    /// Plan what will change when unmarking the stack at this folder.
    PlanUnmark(String),
    /// Turn the stack at this folder into a normal folder.
    Unmark(String),
    /// Turn the normal folder at this path into a stack.
    Mark(String),
    /// Revert a previous mark or unmark operation using its snapshot.
    Undo(Snapshot),
    /// Create a normal folder on disk without `_stack.md`.
    CreateFolder {
        /// Parent folder, if any.
        parent: Option<String>,
        /// Name of the new folder.
        name: String,
    },
}
