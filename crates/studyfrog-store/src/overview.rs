//! What the UI shows about a vault, read from the index (never from the files).

use rusqlite::Connection;
use studyfrog_core::scheduler::{Scheduler, scheduler_from_name};
use studyfrog_core::uid::Uid;
use studyfrog_md::names::decode_names;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};

/// One stack with its item counts and settings.
#[derive(Debug, Clone, PartialEq)]
pub struct StackRow {
    /// Stack identity.
    pub uid: Uid,
    /// Title from `_stack.md`.
    pub title: String,
    /// Vault-relative folder (empty for the vault root).
    pub folder: String,
    /// The folder of the stack this one sits inside, if any.
    pub parent: Option<String>,
    /// The subjects the stack names itself (items and stacks inside it take them when they name none).
    pub subjects: Vec<String>,
    /// Notes in the stack.
    pub notes: usize,
    /// Cards in the stack.
    pub cards: usize,
    /// Questions in the stack.
    pub questions: usize,
    /// The stack's scheduler, `None` for the vault default.
    pub scheduler: Option<Scheduler>,
    /// Daily new-item limit.
    pub daily_new_limit: Option<u32>,
    /// Daily review limit.
    pub daily_review_limit: Option<u32>,
    /// The status the stack states itself; `None` leaves it to its items.
    pub status: Option<studyfrog_core::status::Status>,
}

/// A file or settings problem worth showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// Vault-relative path (a folder for a `_stack.toml`).
    pub path: String,
    /// What is wrong.
    pub message: String,
}

/// The whole picture.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VaultOverview {
    /// Stacks in tree order: a stack comes right before the stacks inside it.
    pub stacks: Vec<StackRow>,
    /// Items outside every stack.
    pub inbox_items: usize,
    /// Files that failed to parse and invalid settings files.
    pub problems: Vec<Problem>,
}

const STACKS_SQL: &str = "
SELECT s.uid, i.title, s.folder, s.scheduler, s.daily_new_limit, s.daily_review_limit,
       (SELECT COUNT(*) FROM item x WHERE x.stack_path = s.folder AND x.kind = 'note'),
       (SELECT COUNT(*) FROM item x WHERE x.stack_path = s.folder AND x.kind = 'card'),
       (SELECT COUNT(*) FROM item x WHERE x.stack_path = s.folder AND x.kind = 'question'),
       i.subjects, i.status
FROM stack_setting s JOIN item i ON i.uid = s.uid
ORDER BY s.folder";

fn stack_row(r: &rusqlite::Row) -> rusqlite::Result<(String, StackRow)> {
    let scheduler: Option<String> = r.get(3)?;
    let count = |i| r.get::<_, i64>(i).map(|n| n as usize);
    let row = StackRow {
        uid: Uid(0),
        title: r.get(1)?,
        folder: r.get(2)?,
        parent: None,
        subjects: subjects_of(r.get::<_, Option<String>>(9)?),
        notes: count(6)?,
        cards: count(7)?,
        questions: count(8)?,
        scheduler: scheduler.as_deref().and_then(scheduler_from_name),
        daily_new_limit: r.get(4)?,
        daily_review_limit: r.get(5)?,
        status: r
            .get::<_, Option<String>>(10)?
            .as_deref()
            .and_then(studyfrog_core::status::status_from_name),
    };
    Ok((r.get(0)?, row))
}

/// The names in the stored `subjects` value (plain or quoted, see `studyfrog_md::names`).
fn subjects_of(list: Option<String>) -> Vec<String> {
    list.as_deref().map_or_else(Vec::new, decode_names)
}

fn stacks(conn: &Connection) -> Result<Vec<StackRow>, StoreError> {
    let mut stmt = conn.prepare(STACKS_SQL).map_err(sql_err)?;
    let rows = stmt.query_map([], stack_row).map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (id, mut stack) = row.map_err(sql_err)?;
        stack.uid = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        out.push(stack);
    }
    set_parents(&mut out);
    out.sort_by(|a, b| a.folder.split('/').cmp(b.folder.split('/')));
    Ok(out)
}

/// Each stack's parent is the stack with the longest folder that contains it.
fn set_parents(stacks: &mut [StackRow]) {
    let folders: Vec<String> = stacks.iter().map(|s| s.folder.clone()).collect();
    for s in stacks.iter_mut() {
        s.parent = folders
            .iter()
            .filter(|f| **f != s.folder && (f.is_empty() || s.folder.starts_with(&format!("{f}/"))))
            .max_by_key(|f| f.len())
            .cloned();
    }
}

fn problems(conn: &Connection) -> Result<Vec<Problem>, StoreError> {
    let sql = "SELECT path, error FROM file WHERE error IS NOT NULL
               UNION ALL SELECT folder, error FROM stack_config WHERE error IS NOT NULL ORDER BY 1";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Problem {
                path: r.get(0)?,
                message: r.get(1)?,
            })
        })
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

/// Read the overview from the index.
pub fn load_overview(conn: &Connection) -> Result<VaultOverview, StoreError> {
    let inbox: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM item WHERE stack_path IS NULL AND kind != 'stack'",
            [],
            |r| r.get(0),
        )
        .map_err(sql_err)?;
    Ok(VaultOverview {
        stacks: stacks(conn)?,
        inbox_items: inbox as usize,
        problems: problems(conn)?,
    })
}

pub use crate::items::{
    IndexedItem, ItemRow, due_items, item_path, linking_to, list_items, load_indexed_items,
    recent_items, resolve_title, search_items, status_counts,
};
pub use crate::link_graph::{GraphNode, LinkGraph, load_link_graph};
