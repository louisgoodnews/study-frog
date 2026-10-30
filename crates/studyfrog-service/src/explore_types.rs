//! The data of the report explorer.

use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_report::explore::{DayPoint, Perf};

/// What to look at.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Scope {
    /// Everything.
    #[default]
    Vault,
    /// A stack (vault-relative folder) and the stacks inside it.
    Stack(String),
    /// One item.
    Item(Uid),
}

/// How far back to look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Period {
    /// The whole history.
    #[default]
    All,
    /// The last this many days (today included).
    Days(u32),
}

/// What a row of the list is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A stack.
    Stack,
    /// An item.
    Item(ItemKind),
}

/// One line of the list: something one level down and how it went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Name.
    pub label: String,
    /// Stack or item.
    pub kind: RowKind,
    /// Where clicking it leads.
    pub target: Scope,
    /// How it went.
    pub perf: Perf,
}

/// One review of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewLine {
    /// When.
    pub at_ms: i64,
    /// The rating (1 to 4).
    pub rating: u8,
}

/// Everything the explorer shows for a scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explorer {
    /// What is shown.
    pub scope: Scope,
    /// The way down: a label and a scope for each step, the last being the scope itself.
    pub crumbs: Vec<(String, Scope)>,
    /// The scope as a whole.
    pub perf: Perf,
    /// The last days.
    pub chart: Vec<DayPoint>,
    /// Stacks and items one level down.
    pub rows: Vec<Row>,
    /// How each subject did (whole vault only).
    pub subjects: Vec<(String, Perf)>,
    /// The reviews of an item, newest first (item scope only).
    pub history: Vec<ReviewLine>,
}
