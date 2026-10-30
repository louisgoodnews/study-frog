//! The spaced order: due reviews (most overdue first), then new items, each within its budget.
//! Items reviewed but not yet due are left out.

use studyfrog_core::time::UnixMs;

use crate::queue::candidate::{Budget, Candidate, is_due, is_reviewed};

fn cap<T>(mut items: Vec<T>, limit: Option<u32>) -> Vec<T> {
    if let Some(n) = limit {
        items.truncate(n as usize);
    }
    items
}

/// Arrange admitted candidates for a spaced run.
pub fn spaced_queue<'a>(
    admitted: &[&'a Candidate<'a>],
    now: UnixMs,
    budget: Budget,
) -> Vec<&'a Candidate<'a>> {
    let mut due: Vec<&Candidate> = admitted
        .iter()
        .copied()
        .filter(|c| is_due(c, now))
        .collect();
    due.sort_by_key(|c| (c.due, c.position, c.uid));
    let mut new: Vec<&Candidate> = admitted
        .iter()
        .copied()
        .filter(|c| !is_reviewed(c))
        .collect();
    new.sort_by_key(|c| (c.position, c.uid));
    let mut queue = cap(due, budget.reviews);
    queue.extend(cap(new, budget.new_items));
    queue
}
