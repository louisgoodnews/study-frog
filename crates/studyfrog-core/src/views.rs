//! How long an item was looked at or worked on, as recorded by the item page (`docs/specs/familiarity.md`).

use crate::time::UnixMs;
use crate::uid::Uid;

/// What the time was spent on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewKind {
    /// The item page was open.
    View,
    /// The item page was open in a session in which the item was changed or saved.
    Edit,
}

/// The word stored for a kind.
pub fn view_kind_name(kind: ViewKind) -> &'static str {
    match kind {
        ViewKind::View => "view",
        ViewKind::Edit => "edit",
    }
}

/// The kind with this stored word.
pub fn view_kind_from_name(name: &str) -> Option<ViewKind> {
    [ViewKind::View, ViewKind::Edit]
        .into_iter()
        .find(|k| view_kind_name(*k) == name)
}

/// One recorded visit to an item. Idle time is not part of `active_ms`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewEvent {
    /// The item.
    pub item: Uid,
    /// What the time was spent on.
    pub kind: ViewKind,
    /// When the visit ended.
    pub at: UnixMs,
    /// Active time in milliseconds.
    pub active_ms: u32,
    /// How long the item's text should take to read at the user's speed, if known.
    pub expected_ms: Option<u32>,
}
