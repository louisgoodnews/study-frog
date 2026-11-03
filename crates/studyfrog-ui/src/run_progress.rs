//! How far a rehearsal run has come: the items done against the items still open. Pure.

use studyfrog_rehearsal::session::Session;

/// Items done and items still open in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Items rated or skipped (left behind the cursor).
    pub done: usize,
    /// Items still open: the one on screen, even when its answer is showing, and every one after it,
    /// including items put back at the end to be repeated.
    pub open: usize,
    /// How many of the open and done items are repeats of an item that was rated Again.
    pub repeats: usize,
}

/// The progress of `s`. The list grows when an item is put back, so the bar can step back, and says why.
pub fn progress(s: &Session) -> Progress {
    let total = s.queue.len();
    let done = s.cursor.min(total);
    Progress {
        done,
        open: total - done,
        repeats: s.requeues.values().map(|n| usize::from(*n)).sum(),
    }
}

/// The fraction done, 0 to 1 (an empty run counts as done).
pub fn fraction(p: Progress) -> f32 {
    let total = p.done + p.open;
    if total == 0 {
        1.0
    } else {
        p.done as f32 / total as f32
    }
}

/// The text on the bar: `3 done · 5 left`, with the repeats named when there are any.
pub fn text(p: Progress) -> String {
    let base = format!("{} done · {} left", p.done, p.open);
    match p.repeats {
        0 => base,
        1 => format!("{base} (1 repeat)"),
        n => format!("{base} ({n} repeats)"),
    }
}
