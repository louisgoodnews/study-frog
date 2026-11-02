//! The stacks inside a stack as a tree, for the Sub-stacks tab and the Summary.

use std::collections::HashMap;

use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::date_to_days;
use studyfrog_core::status::{Status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::due_dates::stack_statuses;
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::ItemMeta;
use studyfrog_store::overview::StackRow;
use studyfrog_store::sessions::load_sessions;

use crate::fam_stacks::stack_percents;
use crate::familiarity::pick_exam;
use crate::stack_summary::{inside, status_of};

const DAY_MS: i64 = 86_400_000;

/// A stack inside the one shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubStack {
    /// Its folder.
    pub folder: String,
    /// Its title.
    pub title: String,
    /// Its status (the one it states, else new).
    pub status: Status,
    /// Items done.
    pub done: usize,
    /// Items in all.
    pub total: usize,
    /// Its familiarity in percent, if any item has a history.
    pub percent: Option<u8>,
    /// How deep it is below the stack shown (1: directly inside it).
    pub depth: usize,
    /// Its exam or due date as a day counted from 1970-01-01, the next one if there is one ahead.
    pub due_day: Option<i64>,
}

fn own_deadlines(conn: &rusqlite::Connection) -> Result<HashMap<Uid, Vec<i64>>, StoreError> {
    let mut by: HashMap<Uid, Vec<i64>> = HashMap::new();
    for s in load_sessions(conn)? {
        if let (true, Some(stack)) = (
            matches!(s.kind, EventKind::Exam | EventKind::Deadline),
            s.stack,
        ) {
            by.entry(stack)
                .or_default()
                .push(date_to_days(s.start.date));
        }
    }
    Ok(by)
}

/// Every stack inside `folder`, in tree order, with its roll-up.
pub(crate) fn tree(
    stacks: &[StackRow],
    folder: &str,
    members: &[(&Uid, &ItemMeta)],
    conn: &rusqlite::Connection,
    now: UnixMs,
) -> Result<Vec<SubStack>, StoreError> {
    let own = stack_statuses(conn)?;
    let percents = stack_percents(conn, now)?;
    let deadlines = own_deadlines(conn)?;
    let prefix = format!("{folder}/");
    Ok(stacks
        .iter()
        .filter(|s| s.folder.starts_with(&prefix))
        .map(|s| {
            let inner: Vec<_> = members
                .iter()
                .filter(|(_, m)| m.stack.as_deref().is_some_and(|f| inside(f, &s.folder)))
                .collect();
            let status = own
                .get(&s.folder)
                .and_then(|w| w.as_deref())
                .and_then(status_from_name)
                .unwrap_or(Status::New);
            let days = deadlines.get(&s.uid).cloned().unwrap_or_default();
            SubStack {
                folder: s.folder.clone(),
                title: s.title.clone(),
                status,
                done: inner
                    .iter()
                    .filter(|(_, m)| status_of(m) == Status::Done)
                    .count(),
                total: inner.len(),
                percent: percents.get(&s.folder).copied().flatten(),
                depth: s.folder[prefix.len()..].matches('/').count() + 1,
                due_day: pick_exam(&days, now.0.div_euclid(DAY_MS)),
            }
        })
        .collect())
}
