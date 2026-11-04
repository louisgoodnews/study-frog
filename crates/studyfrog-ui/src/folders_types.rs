//! Data types for the Folders view (Obsidian-like file explorer, TASK-0204).

use std::collections::BTreeSet;
use studyfrog_service::folder_ops::{DeletePlan, MovePlan};

/// Tree sorting order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Alphabetical (A to Z).
    #[default]
    NameAsc,
    /// Reverse alphabetical (Z to A).
    NameDesc,
    /// Modified time (newest first).
    MtimeDesc,
    /// Modified time (oldest first).
    MtimeAsc,
}

/// The kind of entry in the vault tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// A directory.
    Folder,
    /// A markdown note or card.
    Markdown,
    /// A PDF document.
    Pdf,
    /// An image asset (.png, .jpg, etc.).
    Image,
    /// Another recognized attachment.
    Other(String),
}

/// A scanned or derived filesystem entry in the vault.
#[derive(Debug, Clone, PartialEq)]
pub struct FolderEntry {
    /// Vault-relative path with `/` separators.
    pub rel_path: String,
    /// File or folder name.
    pub name: String,
    /// Whether this entry is a folder.
    pub is_folder: bool,
    /// Whether this folder contains a `_stack.md`.
    pub is_stack: bool,
    /// Parent folder relative path (`""` for root entries).
    pub parent: String,
    /// Nesting depth (0 for root entries).
    pub depth: usize,
    /// Entry kind.
    pub kind: EntryKind,
    /// Modification time in nanoseconds since Unix epoch.
    pub mtime_ns: i64,
}

/// A row rendered in the virtualized tree view.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleRow {
    /// Vault-relative path.
    pub rel_path: String,
    /// Display name (extensions like `.md` stripped for markdown notes).
    pub display_name: String,
    /// Whether this row is a folder.
    pub is_folder: bool,
    /// Whether this is a stack folder.
    pub is_stack: bool,
    /// Nesting depth for indentation.
    pub depth: usize,
    /// Entry kind.
    pub kind: EntryKind,
    /// Whether the folder is currently expanded.
    pub expanded: bool,
}

/// Popover form state for creating a new folder.
#[derive(Debug, Clone, PartialEq)]
pub struct NewFolderForm {
    /// Target parent folder.
    pub parent: String,
    /// Typed name.
    pub name: String,
    /// Whether to mark as a stack folder.
    pub is_stack: bool,
}

/// Complete state of the Folders file explorer view.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FoldersState {
    /// All discovered files and folders.
    pub entries: Vec<FolderEntry>,
    /// Set of expanded folder paths.
    pub expanded: BTreeSet<String>,
    /// Path of currently selected entry, if any.
    pub selected: Option<String>,
    /// Active sort order.
    pub sort: SortOrder,
    /// Target path and new name when renaming in place: `(path, text)`.
    pub renaming: Option<(String, String)>,
    /// State of the new-folder popover, if open.
    pub new_folder: Option<NewFolderForm>,
    /// Target for the "Move to..." picker: `(source_path, selected_destination)`.
    pub move_target: Option<(String, String)>,
    /// Pending cross-stack move awaiting confirmation.
    pub confirm_move: Option<Box<MovePlan>>,
    /// Pending delete awaiting confirmation.
    pub confirm_delete: Option<Box<DeletePlan>>,
}

/// Asynchronous effect for folder operations.
#[derive(Debug, Clone, PartialEq)]
pub enum FolderEffect {
    /// Scan the vault filesystem for files and folders.
    Scan,
    /// Create a new markdown note in a folder.
    CreateNote {
        /// Vault-relative folder.
        folder: String,
        /// Title for the note.
        title: String,
    },
    /// Create a folder on disk, optionally marking it as a stack.
    CreateFolder {
        /// Parent folder.
        parent: String,
        /// Folder name.
        name: String,
        /// Whether to create `_stack.md`.
        is_stack: bool,
    },
    /// Rename a file or folder.
    Rename {
        /// Source path.
        src: String,
        /// New name within parent directory.
        new_name: String,
    },
    /// Plan moving a path to a destination directory.
    PlanMove {
        /// Source path.
        src: String,
        /// Destination folder.
        dst_folder: String,
    },
    /// Execute an approved move plan.
    ExecuteMove(Box<MovePlan>),
    /// Plan deleting a file or folder.
    PlanDelete(String),
    /// Execute an approved delete plan.
    ExecuteDelete(Box<DeletePlan>),
    /// Mark an existing folder as a stack.
    MarkStack(String),
}

/// Messages for user interactions in the Folders view.
#[derive(Debug, Clone, PartialEq)]
pub enum FoldersMsg {
    /// Request a fresh scan of the vault.
    Scan,
    /// File scan arrived from the worker.
    Scanned(Result<Vec<FolderEntry>, String>),
    /// Select an entry by relative path.
    Select(String),
    /// Toggle folder expanded/collapsed.
    ToggleExpand(String),
    /// Expand all folders.
    ExpandAll,
    /// Collapse all folders.
    CollapseAll,
    /// Change sort order.
    SetSort(SortOrder),
    /// Start creating a new note in selected folder.
    StartNewNote,
    /// Open the new folder popover.
    StartNewFolder,
    /// Update fields in the new folder popover.
    EditNewFolder(String, bool),
    /// Submit the new folder creation.
    CommitNewFolder,
    /// Cancel the new folder popover.
    CancelNewFolder,
    /// Start renaming an entry in place.
    StartRename(String),
    /// Update text in inline rename input.
    EditRename(String),
    /// Commit inline rename.
    CommitRename,
    /// Cancel inline rename.
    CancelRename,
    /// Open "Move to..." dialog for an entry.
    AskMove(String),
    /// Change selected destination folder in "Move to..." dialog.
    SetMoveTarget(String),
    /// Submit "Move to..." dialog.
    CommitMove,
    /// Result of planning a move.
    MovePlanReady(Result<MovePlan, String>),
    /// Confirm and execute cross-stack move.
    ConfirmMove,
    /// Cancel cross-stack move.
    CancelMove,
    /// Request deletion of an entry.
    AskDelete(String),
    /// Result of planning deletion.
    DeletePlanReady(Result<DeletePlan, String>),
    /// Confirm and execute deletion.
    ConfirmDelete,
    /// Cancel deletion.
    CancelDelete,
    /// Open a file in its respective editor/viewer.
    Open(String),
    /// Keyboard navigation shortcut.
    KeyAction(FoldersKey),
    /// Operation completed, refresh overview.
    OpDone(Result<studyfrog_service::session::VaultInfo, String>),
}

/// Keyboard navigation actions in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldersKey {
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Enter key.
    Enter,
    /// F2 key.
    F2,
    /// Delete or Backspace key.
    Delete,
}
