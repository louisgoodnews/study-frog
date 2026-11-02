//! What the Summary tab of a stack shows: counts of the last and next seven days, the spread of statuses, priorities and
//! types, the recent activity and the sub-stacks. Read when the tab opens, never before.

use std::collections::HashMap;

use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::{PRIORITIES, Priority};
use studyfrog_core::status::{STATUSES, Status, effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::{ItemMeta, load_item_meta};
use studyfrog_store::overview::{load_indexed_items, load_overview};
use studyfrog_store::reviews::load_all_reviews_timed;

pub use crate::stack_tree::SubStack;
use crate::stack_tree::tree;

const DAY_MS: i64 = 86_400_000;
/// Days the cards look back and ahead.
pub const WINDOW_DAYS: i64 = 7;
/// Most entries of the activity feed.
pub const FEED: usize = 20;

/// What happened to an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Happened {
    /// It was created.
    Created,
    /// Its file was edited.
    Edited,
    /// It was rehearsed.
    Rehearsed,
}

/// One line of the feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// What happened.
    pub what: Happened,
    /// When, in Unix milliseconds.
    pub at_ms: i64,
}

/// The Summary of a stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackSummary {
    /// Items that are done and were edited in the last seven days.
    pub completed: usize,
    /// Items edited in the last seven days.
    pub updated: usize,
    /// Items created in the last seven days.
    pub created: usize,
    /// Items due in the next seven days.
    pub due_soon: usize,
    /// Items per status, in the order of [`STATUSES`].
    pub statuses: [usize; 5],
    /// Items per priority, lowest first.
    pub priorities: [usize; 5],
    /// Notes, cards and questions.
    pub types: [usize; 3],
    /// The newest changes.
    pub activity: Vec<Activity>,
    /// The stacks inside this one, directly.
    pub substacks: Vec<SubStack>,
    /// Every stack inside this one, in tree order.
    pub tree: Vec<SubStack>,
}

pub(crate) fn inside(folder: &str, stack: &str) -> bool {
    folder == stack || folder.starts_with(&format!("{stack}/"))
}

pub(crate) fn status_of(m: &ItemMeta) -> Status {
    effective_status(
        m.status.as_deref().and_then(status_from_name),
        m.reviews > 0,
    )
}

fn feed(items: &[(&Uid, &ItemMeta)], last: &HashMap<Uid, i64>) -> Vec<Activity> {
    let mut all = Vec::new();
    for (uid, m) in items {
        let mut add = |what, at: Option<i64>| {
            if let Some(at_ms) = at {
                all.push(Activity {
                    uid: **uid,
                    title: m.title.clone(),
                    what,
                    at_ms,
                });
            }
        };
        add(Happened::Created, m.created_ms);
        add(
            Happened::Edited,
            m.updated_ms.filter(|u| Some(*u) != m.created_ms),
        );
        add(Happened::Rehearsed, last.get(uid).copied());
    }
    all.sort_by(|a, b| b.at_ms.cmp(&a.at_ms).then(a.uid.cmp(&b.uid)));
    all.truncate(FEED);
    all
}

/// The summary of the stack at `folder`, with its sub-stacks' items included.
pub fn stack_summary(
    conn: &rusqlite::Connection,
    folder: &str,
    now: UnixMs,
) -> Result<StackSummary, StoreError> {
    let meta = load_item_meta(conn)?;
    let members: Vec<(&Uid, &ItemMeta)> = meta
        .iter()
        .filter(|(_, m)| m.stack.as_deref().is_some_and(|f| inside(f, folder)))
        .collect();
    let priority: HashMap<Uid, Priority> = load_indexed_items(conn)?
        .into_iter()
        .map(|i| (i.uid, i.priority))
        .collect();
    let last: HashMap<Uid, i64> = load_all_reviews_timed(conn)?
        .into_iter()
        .filter_map(|(uid, v)| v.iter().map(|(r, _)| r.at.0).max().map(|t| (uid, t)))
        .collect();
    let (back, ahead) = (now.0 - WINDOW_DAYS * DAY_MS, now.0 + WINDOW_DAYS * DAY_MS);
    let mut s = StackSummary {
        completed: 0,
        updated: 0,
        created: 0,
        due_soon: 0,
        statuses: [0; 5],
        priorities: [0; 5],
        types: [0; 3],
        activity: feed(&members, &last),
        substacks: Vec::new(),
        tree: Vec::new(),
    };
    for (uid, m) in &members {
        let status = status_of(m);
        let recent = |t: Option<i64>| t.is_some_and(|t| t >= back);
        s.updated += usize::from(recent(m.updated_ms));
        s.created += usize::from(recent(m.created_ms));
        s.completed += usize::from(status == Status::Done && recent(m.updated_ms));
        s.due_soon += usize::from(
            m.due_ms.is_some_and(|d| d >= now.0 && d <= ahead)
                && !matches!(status, Status::Done | Status::Paused),
        );
        if let Some(i) = STATUSES.iter().position(|x| *x == status) {
            s.statuses[i] += 1;
        }
        let p = priority.get(*uid).copied().unwrap_or_default();
        if let Some(i) = PRIORITIES.iter().position(|x| *x == p) {
            s.priorities[i] += 1;
        }
        let t = match m.kind {
            ItemKind::Note => 0,
            ItemKind::Card => 1,
            ItemKind::Question => 2,
        };
        s.types[t] += 1;
    }
    let stacks = load_overview(conn)?.stacks;
    s.tree = tree(&stacks, folder, &members, conn, now)?;
    s.substacks = s.tree.iter().filter(|t| t.depth == 1).cloned().collect();
    Ok(s)
}
