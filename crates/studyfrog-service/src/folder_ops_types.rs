//! Data types for folder and file operation plans and undo snapshots.

use studyfrog_core::uid::Uid;
use studyfrog_store::schedule::{StoredMem, StoredSm2};

/// Pure plan to create a new folder inside the vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateFolderPlan {
    /// Vault-relative path of the folder to create.
    pub path: String,
}

/// A planned rewrite of markdown links in one vault file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkRewrite {
    /// Vault-relative path of the file being rewritten.
    pub path: String,
    /// Content before the rewrite.
    pub old_content: String,
    /// Content after the rewrite.
    pub new_content: String,
}

/// Pure plan to rename a folder or file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenamePlan {
    /// Vault-relative source path.
    pub src: String,
    /// Vault-relative destination path.
    pub dst: String,
    /// Whether the target is a folder.
    pub is_folder: bool,
    /// All files affected by the rename (source and destination relative paths).
    pub files: Vec<(String, String)>,
    /// Markdown files with picture/attachment links rewritten.
    pub link_rewrites: Vec<LinkRewrite>,
}

/// Dropped memory tuple for an unreferenced schedule.
pub type DroppedMemory = (Uid, Uid, Option<StoredMem>, Option<StoredSm2>);

/// Pure plan to move a folder or file, with cross-stack and memory tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct MovePlan {
    /// Vault-relative source path.
    pub src: String,
    /// Vault-relative destination path.
    pub dst: String,
    /// Whether the target is a folder.
    pub is_folder: bool,
    /// All files being moved `(from_rel, to_rel)`.
    pub files: Vec<(String, String)>,
    /// Whether the move crosses a stack boundary.
    pub is_cross_stack: bool,
    /// Source stack folder, or `None` for Inbox.
    pub source_stack: Option<String>,
    /// Destination stack folder, or `None` for Inbox.
    pub destination_stack: Option<String>,
    /// Source stack title (e.g. "French" or "Inbox").
    pub source_stack_title: String,
    /// Destination stack title (e.g. "German" or "Inbox").
    pub destination_stack_title: String,
    /// How many learning items change their primary stack.
    pub items_changing_primary_stack: usize,
    /// How many unreferenced SRS schedules are dropped.
    pub schedules_dropped: usize,
    /// Unreferenced memory states dropped during move.
    pub dropped_memory: Vec<DroppedMemory>,
    /// Frontmatter adjustments (e.g. removing new primary stack from `referenced_by`).
    pub doc_updates: Vec<LinkRewrite>,
    /// Link rewrites across vault markdown files.
    pub link_rewrites: Vec<LinkRewrite>,
}

/// Pure plan to delete a folder or file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletePlan {
    /// Vault-relative path of target to delete.
    pub target: String,
    /// Whether the target is a folder.
    pub is_folder: bool,
    /// Number of files being deleted.
    pub files_count: usize,
    /// Number of learning items being deleted.
    pub items_count: usize,
    /// Number of stacks being deleted.
    pub stacks_count: usize,
    /// All relative file paths to delete.
    pub files_to_delete: Vec<String>,
}

/// Individual atomic step for undoing a folder operation.
#[derive(Debug, Clone, PartialEq)]
pub enum UndoAction {
    /// Delete a directory if it was created.
    DeleteDir(String),
    /// Delete a file if it was created.
    DeleteFile(String),
    /// Move a path back from destination to source.
    MovePath {
        /// Current location on disk.
        from: String,
        /// Original location on disk.
        to: String,
    },
    /// Restore file content.
    WriteFile(String, String),
    /// Re-insert dropped memory state.
    RestoreMemory {
        /// Item identity.
        item_uid: Uid,
        /// Stack identity.
        stack_uid: Uid,
        /// FSRS memory state.
        mem: Option<StoredMem>,
        /// SM-2 memory state.
        sm2: Option<StoredSm2>,
    },
    /// Recreate a deleted file.
    RecreateFile(String, Vec<u8>),
}

/// Snapshot of operations performed, enabling full atomic rollback and undo.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FolderSnapshot(pub Vec<UndoAction>);
