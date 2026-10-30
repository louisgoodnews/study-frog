//! The spaced order: due reviews (most overdue first), then new items, each within its budget.
//! Items reviewed but not yet due are left out.

use std::collections::HashMap;

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

use std::cmp::Reverse;

use studyfrog_core::priority::Priority;

use crate::queue::candidate::{Budget, Budgets, Candidate, is_due, is_reviewed};

const HOUR_MS: i64 = 3_600_000;

/// How early (positive) or late (negative) an item of this priority is suggested: a high-priority item
/// is due half a day early, the highest a day early, the lowest only half a day after its due time.
pub fn lead_ms(p: Priority) -> i64 {
    match p {
        Priority::Highest => 24 * HOUR_MS,
        Priority::High => 12 * HOUR_MS,
        Priority::Medium | Priority::Low => 0,
        Priority::Lowest => -12 * HOUR_MS,
    }
}

fn cap<T>(mut items: Vec<T>, limit: Option<u32>) -> Vec<T> {
    if let Some(n) = limit {
        items.truncate(n as usize);
    }
    items
}

/// Keep items in order while their stack still has allowance (`pick` selects the review or new
/// allowance). Stacks without an entry are unlimited.
fn within_stacks<'a>(
    items: Vec<&'a Candidate<'a>>,
    per_stack: &HashMap<Uid, Budget>,
    pick: fn(&Budget) -> Option<u32>,
) -> Vec<&'a Candidate<'a>> {
    let mut taken: HashMap<Uid, u32> = HashMap::new();
    let mut keep = |c: &&Candidate| {
        let Some(limit) = per_stack.get(&c.stack).and_then(pick) else {
            return true;
        };
        let used = taken.entry(c.stack).or_insert(0);
        *used += 1;
        *used <= limit
    };
    items.into_iter().filter(|c| keep(c)).collect()
}

/// Arrange admitted candidates for a spaced run.
pub fn spaced_queue<'a>(
    admitted: &[&'a Candidate<'a>],
    now: UnixMs,
    budgets: &Budgets,
) -> Vec<&'a Candidate<'a>> {
    let mut due: Vec<&Candidate> = admitted
        .iter()
        .copied()
        .filter(|c| is_due(c, UnixMs(now.0 + lead_ms(c.priority))))
        .collect();
    due.sort_by_key(|c| (Reverse(c.priority), c.due, c.position, c.uid));
    let mut new: Vec<&Candidate> = admitted
        .iter()
        .copied()
        .filter(|c| !is_reviewed(c))
        .collect();
    new.sort_by_key(|c| (Reverse(c.priority), c.position, c.uid));
    let mut queue = cap(
        within_stacks(due, &budgets.per_stack, |b| b.reviews),
        budgets.global.reviews,
    );
    queue.extend(cap(
        within_stacks(new, &budgets.per_stack, |b| b.new_items),
        budgets.global.new_items,
    ));
    queue
}
