//! The calendar's planned reviews: read from the schedule, and moved to another day without touching the memory state.

use std::collections::HashMap;

use studyfrog_calendar::review_plan::{PlanEntry, PlanItem, plan_entries};
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_core::status::{effective_status, skips_schedule, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::due_dates::{Dues, load_dues, set_dues, stack_statuses};
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{StackRow, load_overview};

use crate::library_rows::exams;
use crate::session::{ServiceError, VaultSession};
use crate::timezone::{local_day_of, local_day_start};

/// An item's due dates before a move, to put back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prior {
    /// The item.
    pub item: Uid,
    /// What its states said.
    pub dues: Dues,
}

/// What a move did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    /// How many items now fall due on the new day.
    pub moved: usize,
    /// How to take it back.
    pub undo: Vec<Prior>,
    /// Set when the new day is after an exam or deadline of a stack the items sit in.
    pub warning: Option<String>,
}

/// Whether the stack at `folder` or one above it is done or paused.
fn stack_skips(statuses: &HashMap<String, Option<String>>, folder: &str) -> bool {
    let mut upto = String::new();
    folder.split('/').any(|part| {
        upto = if upto.is_empty() {
            part.to_owned()
        } else {
            format!("{upto}/{part}")
        };
        let stated = statuses.get(&upto).and_then(|s| s.as_deref());
        stated
            .and_then(status_from_name)
            .is_some_and(skips_schedule)
    })
}

fn uid_of(stacks: &[StackRow], folder: Option<&String>) -> Option<Uid> {
    let folder = folder?;
    stacks.iter().find(|s| s.folder == *folder).map(|s| s.uid)
}

/// The planned reviews from `from_day` to `to_day` (local days counted from 1970-01-01).
pub fn plan_reviews(
    session: &VaultSession,
    days: (i64, i64),
) -> Result<Vec<PlanEntry>, ServiceError> {
    let conn = &session.conn;
    let store = ServiceError::Store;
    let stacks = load_overview(conn).map_err(store)?.stacks;
    let statuses = stack_statuses(conn).map_err(ServiceError::Store)?;
    let mut sizes: HashMap<Option<Uid>, usize> = HashMap::new();
    let mut planned = Vec::new();
    for (uid, meta) in load_item_meta(conn).map_err(ServiceError::Store)? {
        let stack = uid_of(&stacks, meta.stack.as_ref());
        *sizes.entry(stack).or_default() += 1;
        let Some(due) = meta.due_ms else { continue };
        let status = effective_status(meta.status.as_deref().and_then(status_from_name), true);
        planned.push(PlanItem {
            uid,
            stack,
            due_day: local_day_of(UnixMs(due)),
            status,
            stack_skips: meta
                .stack
                .as_deref()
                .is_some_and(|f| stack_skips(&statuses, f)),
        });
    }
    Ok(plan_entries(&planned, &sizes, days))
}

/// The warning for putting items of these stacks on `to_day`, if an exam or deadline comes before it.
fn warning_for(
    session: &VaultSession,
    items: &[Uid],
    to_day: i64,
    today: i64,
) -> Result<Option<String>, ServiceError> {
    let conn = &session.conn;
    let stacks = load_overview(conn).map_err(ServiceError::Store)?.stacks;
    let meta = load_item_meta(conn).map_err(ServiceError::Store)?;
    let by_folder = exams(conn, &stacks, today).map_err(ServiceError::Store)?;
    let mut hits: Vec<(String, i64)> = Vec::new();
    for folder in items.iter().filter_map(|u| meta.get(u)?.stack.as_ref()) {
        let exam = by_folder
            .get(folder)
            .copied()
            .filter(|d| *d >= today && *d < to_day);
        let title = stacks
            .iter()
            .find(|s| s.folder == *folder)
            .map(|s| s.title.clone());
        if let (Some(d), Some(t)) = (exam, title)
            && !hits.iter().any(|(x, _)| *x == t)
        {
            hits.push((t, d));
        }
    }
    let said: Vec<String> = hits
        .iter()
        .map(|(t, d)| format!("{t} ({})", format_date(date_from_days(*d))))
        .collect();
    Ok((!said.is_empty())
        .then(|| format!("That is after the exam or due date of {}.", said.join(", "))))
}

/// Make the items due on `to_day` (the start of that local day) and keep everything else of their schedule.
/// Items in no schedule are left out of the count. A move is undone with [`undo_move`].
pub fn move_reviews(
    session: &VaultSession,
    items: &[Uid],
    to_day: i64,
    now: UnixMs,
) -> Result<Moved, ServiceError> {
    let due = local_day_start(to_day)
        .0
        .max(now.0.min(local_day_start(to_day).0));
    let mut undo = Vec::new();
    for item in items {
        let before = load_dues(&session.conn, *item).map_err(ServiceError::Store)?;
        if before.fsrs.is_none() && before.sm2.is_none() {
            continue;
        }
        let moved = Dues {
            fsrs: before.fsrs.map(|_| due),
            sm2: before.sm2.map(|_| due),
        };
        set_dues(&session.conn, *item, moved).map_err(ServiceError::Store)?;
        undo.push(Prior {
            item: *item,
            dues: before,
        });
    }
    let warning = warning_for(session, items, to_day, local_day_of(now))?;
    Ok(Moved {
        moved: undo.len(),
        undo,
        warning,
    })
}

/// Put the due dates back as a move found them.
pub fn undo_move(session: &VaultSession, undo: &[Prior]) -> Result<(), ServiceError> {
    for p in undo {
        set_dues(&session.conn, p.item, p.dues).map_err(ServiceError::Store)?;
    }
    Ok(())
}
