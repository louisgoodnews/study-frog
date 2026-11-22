//! The homework commands of the control protocol (STORY-0190), as plain data.

use studyfrog_calendar::homework::HomeworkPriority;
use studyfrog_calendar::homework_query::HomeworkTab;
use studyfrog_core::uid::Uid;

/// What `homework ...` asks for. An item is named by its ULID.
#[derive(Debug, Clone, PartialEq)]
pub enum HomeworkCmd {
    /// The items as the app last read them.
    List,
    /// Show a status tab.
    Tab(HomeworkTab),
    /// Narrow the list to one subject (a stack folder), or every subject with `None`.
    Subject(Option<String>),
    /// Create an item (the dialog is filled in and saved).
    New(HomeworkNew),
    /// Move an item on to its next status.
    Advance(Uid),
    /// Tick or untick checklist step number `n` (from 1).
    Tick(Uid, usize),
}

/// The fields of a new homework item; what is not given keeps the dialog's default.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HomeworkNew {
    /// The title.
    pub title: String,
    /// The due date, `YYYY-MM-DD`.
    pub due_date: Option<String>,
    /// The due time, `HH:MM`.
    pub due_time: Option<String>,
    /// Estimated minutes.
    pub estimate: Option<u32>,
    /// Priority.
    pub priority: Option<HomeworkPriority>,
    /// The stack folder the item belongs to.
    pub stack: Option<String>,
    /// The checklist steps, in order.
    pub checklist: Vec<String>,
}
