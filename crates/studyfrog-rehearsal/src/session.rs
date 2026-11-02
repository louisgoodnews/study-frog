//! Mutable run state advanced by `step`. Data only.

use std::collections::HashMap;

use studyfrog_core::run::Run;
use studyfrog_core::uid::Uid;

/// Where the current item is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemPhase {
    /// No item is on screen (before the start and after the end).
    NoItem,
    /// The prompt is shown.
    Shown,
    /// The answer has been revealed (flipped or answered).
    Revealed,
}

/// A run in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    /// The run (identity, mode, plan, lifecycle state).
    pub run: Run,
    /// Working queue; grows when items are requeued.
    pub queue: Vec<Uid>,
    /// Index of the current item in `queue`; equals `queue.len()` when exhausted.
    pub cursor: usize,
    /// State of the current item.
    pub phase: ItemPhase,
    /// When the current item was shown.
    pub shown_at_ms: u32,
    /// When the current pause began, if paused.
    pub paused_at_ms: Option<u32>,
    /// Paused time accumulated while the current item was on screen.
    pub paused_in_item_ms: u32,
    /// When the run started, on the run's clock.
    pub started_ms: u32,
    /// All time paused so far, not counting a pause that is going on.
    pub paused_total_ms: u32,
    /// Time of the last accepted input.
    pub last_t_ms: u32,
    /// Sequence number of the next event.
    pub next_seq: u32,
    /// How many times each item has been requeued in this run.
    pub requeues: HashMap<Uid, u8>,
}

/// A fresh session over `run.queue`, not yet started.
pub fn session_new(run: Run) -> Session {
    let queue = run.queue.clone();
    Session {
        run,
        queue,
        cursor: 0,
        phase: ItemPhase::NoItem,
        shown_at_ms: 0,
        paused_at_ms: None,
        paused_in_item_ms: 0,
        started_ms: 0,
        paused_total_ms: 0,
        last_t_ms: 0,
        next_seq: 0,
        requeues: HashMap::new(),
    }
}

/// The item currently on screen.
pub fn current_item(session: &Session) -> Option<Uid> {
    match session.phase {
        ItemPhase::NoItem => None,
        _ => session.queue.get(session.cursor).copied(),
    }
}
