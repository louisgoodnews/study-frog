//! Queue-builder input: what the shell knows about each studiable item. Plain data.

use studyfrog_core::memory::MemState;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

/// One item that may enter a queue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate<'a> {
    /// Item identity.
    pub uid: Uid,
    /// Note, card or question.
    pub kind: ItemKind,
    /// Owning stack.
    pub stack: Uid,
    /// Tags, borrowed from the library.
    pub tags: &'a [String],
    /// Stable position in the library (path order); the base order of every sequence.
    pub position: u32,
    /// FSRS state, if the item has been scheduled with FSRS.
    pub memory: Option<MemState>,
    /// When the item is next due; `None` for new items.
    pub due: Option<UnixMs>,
    /// When the item was last reviewed; `None` for new items.
    pub last_review: Option<UnixMs>,
}

/// Daily allowances for the spaced order; `None` means unlimited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// New items still allowed today.
    pub new_items: Option<u32>,
    /// Due reviews still allowed today.
    pub reviews: Option<u32>,
}

/// No daily limits.
pub fn budget_unlimited() -> Budget {
    Budget {
        new_items: None,
        reviews: None,
    }
}

/// True once the item has been reviewed at least once (with any scheduler).
pub fn is_reviewed(c: &Candidate) -> bool {
    c.memory.is_some() || c.last_review.is_some() || c.due.is_some()
}

/// A reviewed item is due when its due time has come (or was never recorded).
pub fn is_due(c: &Candidate, now: UnixMs) -> bool {
    is_reviewed(c) && c.due.is_none_or(|d| d <= now)
}
