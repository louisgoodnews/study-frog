//! The Backlog tab of a stack: what is coming up and what is not planned yet. Pure.
//!
//! *Up next* holds the active items (learning or reviewing) and any item due within [`HORIZON_DAYS`]; *not yet planned* holds
//! new items and those on hold. Done items are in neither. Moving an item to Up next makes it Learning; moving it back puts
//! it on hold, so the schedule leaves it alone.

use studyfrog_core::status::{STATUSES, Status};

use crate::library_ui::LibraryState;

/// Days ahead that count as "up next" for an item that is not active.
pub const HORIZON_DAYS: i64 = 14;

/// The two lists, as indices into the library's items.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Backlog {
    /// Active or soon due, the soonest due first.
    pub up_next: Vec<usize>,
    /// New or on hold.
    pub not_planned: Vec<usize>,
}

fn in_stack(chain: &[String], folder: &str) -> bool {
    chain.iter().step_by(2).any(|s| s == folder)
}

/// The backlog of the stack at `folder` (its sub-stacks included) on day `today`.
pub fn backlog(lib: &LibraryState, folder: &str, today: i64) -> Backlog {
    let mut out = Backlog::default();
    for (i, item) in lib.items.iter().enumerate() {
        let row = &item.row;
        if !in_stack(&row.stacks, folder) {
            continue;
        }
        let soon = row.due_day.is_some_and(|d| d <= today + HORIZON_DAYS);
        match row.status {
            Status::Done => {}
            Status::Paused => out.not_planned.push(i),
            Status::Learning | Status::Reviewing => out.up_next.push(i),
            Status::New if soon => out.up_next.push(i),
            Status::New => out.not_planned.push(i),
        }
    }
    out.up_next
        .sort_by_key(|i| (lib.items[*i].row.due_day.unwrap_or(i64::MAX), *i));
    out
}

/// How many of `items` have each status, in the order of [`STATUSES`].
pub fn counts(lib: &LibraryState, items: &[usize]) -> [usize; 5] {
    let mut n = [0; 5];
    for i in items {
        if let Some(k) = STATUSES.iter().position(|s| *s == lib.items[*i].row.status) {
            n[k] += 1;
        }
    }
    n
}

/// The status that moves an item into (`true`) or out of Up next.
pub fn status_for(up_next: bool) -> Status {
    if up_next {
        Status::Learning
    } else {
        Status::Paused
    }
}
