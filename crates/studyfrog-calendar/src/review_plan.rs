//! The reviews the schedule has planned, as calendar entries: one per stack and day, with how many items fall due
//! and how big the stack is. Computed from the schedule every time, never stored. Pure.

use std::collections::HashMap;

use studyfrog_core::status::{Status, skips_schedule};
use studyfrog_core::uid::Uid;

/// An item with its next due day, as the planner needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanItem {
    /// The item.
    pub uid: Uid,
    /// The stack it sits in; `None` for the inbox.
    pub stack: Option<Uid>,
    /// The day it is next due, counted from 1970-01-01 in the local calendar.
    pub due_day: i64,
    /// The item's own status.
    pub status: Status,
    /// A stack above it (or its own) is done or paused, so the whole stack is skipped.
    pub stack_skips: bool,
}

/// What the calendar shows for one stack on one day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntry {
    /// The stack; `None` for items outside any stack.
    pub stack: Option<Uid>,
    /// The day.
    pub day: i64,
    /// The items due.
    pub items: Vec<Uid>,
    /// How many items the stack holds in all.
    pub stack_size: usize,
}

/// The entries from `from_day` to `to_day` (both included), by day and then stack. An item that is already
/// overdue appears on `from_day`; items that are done or paused, or in a stack that is, appear nowhere.
pub fn plan_entries(
    items: &[PlanItem],
    sizes: &HashMap<Option<Uid>, usize>,
    (from_day, to_day): (i64, i64),
) -> Vec<PlanEntry> {
    let mut groups: HashMap<(i64, Option<Uid>), Vec<Uid>> = HashMap::new();
    for item in items {
        if skips_schedule(item.status) || item.stack_skips {
            continue;
        }
        let day = item.due_day.max(from_day);
        if day <= to_day {
            groups.entry((day, item.stack)).or_default().push(item.uid);
        }
    }
    let mut out: Vec<PlanEntry> = groups
        .into_iter()
        .map(|((day, stack), mut items)| {
            items.sort();
            let stack_size = sizes.get(&stack).copied().unwrap_or(items.len());
            PlanEntry {
                stack,
                day,
                items,
                stack_size,
            }
        })
        .collect();
    out.sort_by_key(|e| (e.day, e.stack));
    out
}
