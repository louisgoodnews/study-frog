//! Item status: where a stack, note, card or question stands (`docs/specs/status.md`).

/// The status of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    /// Never rehearsed (a note: never viewed). The default.
    #[default]
    New,
    /// Rehearsed, not yet known well.
    Learning,
    /// Known well, kept fresh by spaced repetition.
    Reviewing,
    /// The user is finished with it; the schedule skips it.
    Done,
    /// Kept but not studied for now; the schedule skips it.
    Paused,
}

/// Every status, in the order a menu lists them.
pub const STATUSES: [Status; 5] = [
    Status::New,
    Status::Learning,
    Status::Reviewing,
    Status::Done,
    Status::Paused,
];

/// The word stored in a file and in the index.
pub fn status_name(s: Status) -> &'static str {
    match s {
        Status::New => "new",
        Status::Learning => "learning",
        Status::Reviewing => "reviewing",
        Status::Done => "done",
        Status::Paused => "paused",
    }
}

/// The status with this stored word.
pub fn status_from_name(name: &str) -> Option<Status> {
    STATUSES
        .into_iter()
        .find(|s| status_name(*s) == name.trim())
}

/// The plain-language name shown to the user.
pub fn status_label(s: Status) -> &'static str {
    match s {
        Status::New => "Not studied yet",
        Status::Learning => "Learning",
        Status::Reviewing => "In review",
        Status::Done => "Complete",
        Status::Paused => "On hold",
    }
}

/// The status in effect: the stored one if there is one, else derived from the history
/// (no review yet is `New`, at least one is `Learning`).
pub fn effective_status(stored: Option<Status>, reviewed: bool) -> Status {
    stored.unwrap_or(if reviewed {
        Status::Learning
    } else {
        Status::New
    })
}

/// Whether the schedule leaves items with this status out (calendar, due queues).
pub fn skips_schedule(s: Status) -> bool {
    matches!(s, Status::Done | Status::Paused)
}
