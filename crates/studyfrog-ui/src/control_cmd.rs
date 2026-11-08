//! The commands of the control protocol (STORY-0124) as plain data, and what each one may change.
//! Parsing is in `control_parse`, the messages each command becomes in `control_plan`.

use std::path::PathBuf;

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_service::create_more::NewKind;

use crate::state::Screen;

/// One request: the caller's id (echoed in the answer) and the command.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    /// Any JSON value the caller chose; `null` when it gave none.
    pub id: serde_json::Value,
    /// What to do.
    pub cmd: Command,
}

/// Everything the protocol can be asked.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// A snapshot of the state.
    State,
    /// The list of commands.
    Help,
    /// Go to a screen.
    Nav(Screen),
    /// Back in the history.
    Back,
    /// Forward in the history.
    Forward,
    /// Open the vault in this folder.
    VaultOpen(String),
    /// Close the vault.
    VaultClose,
    /// An item operation.
    Item(ItemCmd),
    /// A stack operation.
    Stack(StackCmd),
    /// A step of the rehearsal.
    Rehearse(RehearseCmd),
    /// Run a library query and list the matching items.
    Query(String),
    /// The top-bar search (items and PDFs).
    Search(String),
    /// Export the library or a stack.
    Export(ExportArgs),
    /// Import a file, into a stack or where the file says.
    Import {
        /// The file or folder.
        path: String,
        /// The stack folder to import into.
        stack: Option<String>,
    },
    /// Save a picture of the window.
    Screenshot(PathBuf),
    /// Read (and maybe follow) the console log.
    Logs(LogsQuery),
    /// The recent notifications.
    Notices,
    /// Dismiss the status-bar notice.
    Dismiss,
    /// Close the open dialog.
    CloseModal,
    /// Do nothing but wait until background work is done.
    Settle,
}

/// What `item ...` asks for. An item is named by its ULID or its key (`FLASHCARD_12`).
#[derive(Debug, Clone, PartialEq)]
pub enum ItemCmd {
    /// Create a note, card or question.
    Create(NewItemArgs),
    /// Open the item page.
    Open(String),
    /// Change one field and save.
    Edit(String, EditTarget, String),
    /// Delete the item.
    Delete(String),
    /// Give the item a colour, or take it away.
    Color(String, Option<String>),
    /// Move the item into the stack with this folder.
    Move(String, String),
}

/// The fields of a new item.
#[derive(Debug, Clone, PartialEq)]
pub struct NewItemArgs {
    /// Stack, note, card or question.
    pub kind: NewKind,
    /// The title.
    pub title: String,
    /// The body, front, prompt or description.
    pub text: String,
    /// The back or the reference answer.
    pub answer: String,
    /// Tags, comma separated.
    pub tags: String,
    /// The stack folder it goes into.
    pub stack: Option<String>,
}

/// Which field `item edit` changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditTarget {
    /// The title.
    Title,
    /// The tags, comma separated.
    Tags,
    /// A text area by index (0: body, front or prompt; 1: back or reference).
    Field(usize),
}

/// What `stack ...` asks for. A stack is named by its folder.
#[derive(Debug, Clone, PartialEq)]
pub enum StackCmd {
    /// Create a stack (inside a parent stack).
    Create(String, Option<String>),
    /// Open the stack page.
    Open(String),
    /// Move the stack into another one (`""`: the top level).
    Move(String, String),
    /// The stack's scheduling settings, and its page.
    Settings(String),
}

/// What `rehearse ...` asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum RehearseCmd {
    /// Go to the setup and fill it in.
    Setup(SetupArgs),
    /// Start the run.
    Start,
    /// Reveal the answer.
    Flip,
    /// Submit a typed answer.
    Answer(String),
    /// Rate the revealed item.
    Rate(Rating),
    /// Mark how hard the item is.
    Mark(Difficulty),
    /// Pass on the item.
    Skip,
    /// Pause the timer.
    Pause,
    /// Resume after a pause.
    Resume,
    /// End the run early.
    Stop,
    /// Leave the finished run for the setup.
    Leave,
    /// Write the answer field without submitting it.
    Type(String),
    /// Submit (and in an exam lock) what the answer field holds.
    Submit,
    /// Leave a finished exam's result for the setup; refused while the exam is running.
    Finish,
}

/// The setup of a run; what is not given keeps its value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetupArgs {
    /// `spaced`, `all`, `shuffled`, `hardest` or `stalest`.
    pub preset: Option<crate::setup::Preset>,
    /// Flip cards or type answers.
    pub mode: Option<studyfrog_core::run::RunMode>,
    /// At most this many items (`""`: no limit).
    pub limit: Option<String>,
    /// Only these stack folders.
    pub stacks: Vec<String>,
}

/// What `export` asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportArgs {
    /// The file format.
    pub format: crate::transfer_types::Format,
    /// The folder written to.
    pub dest: String,
    /// One stack instead of the library.
    pub stack: Option<String>,
    /// Include the scheduling (JSON only).
    pub scheduling: bool,
}

/// What `logs` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogsQuery {
    /// How many of the last entries.
    pub tail: usize,
    /// Keep sending entries as they are written.
    pub follow: bool,
    /// Only entries at least this serious (`info`, `warn`, `error`).
    pub level: Option<String>,
}

pub use crate::control_policy::{Class, ControlPolicy, allowed, class_of, policy_default};
