//! What the UI shows about a vault, read from the index (never from the files).

use rusqlite::Connection;
use studyfrog_core::scheduler::{Scheduler, scheduler_from_name};
use studyfrog_core::uid::Uid;
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
    /// Stacks, ordered by folder.
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
       (SELECT COUNT(*) FROM item x WHERE x.stack_path = s.folder AND x.kind = 'question')
FROM stack_setting s JOIN item i ON i.uid = s.uid
ORDER BY s.folder";

fn stack_row(r: &rusqlite::Row) -> rusqlite::Result<(String, StackRow)> {
    let scheduler: Option<String> = r.get(3)?;
    let count = |i| r.get::<_, i64>(i).map(|n| n as usize);
    let row = StackRow {
        uid: Uid(0),
        title: r.get(1)?,
        folder: r.get(2)?,
        notes: count(6)?,
        cards: count(7)?,
        questions: count(8)?,
        scheduler: scheduler.as_deref().and_then(scheduler_from_name),
        daily_new_limit: r.get(4)?,
        daily_review_limit: r.get(5)?,
    };
    Ok((r.get(0)?, row))
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
    Ok(out)
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

/// One item of a stack (or of the inbox) as the UI lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRow {
    /// Item identity.
    pub uid: Uid,
    /// Title.
    pub title: String,
    /// Note, card or question.
    pub kind: studyfrog_core::plan::ItemKind,
    /// Vault-relative path of the file.
    pub path: String,
}

/// Items directly in `folder` (`None` for the inbox), without the stack file, ordered by path.
pub fn list_items(conn: &Connection, folder: Option<&str>) -> Result<Vec<ItemRow>, StoreError> {
    let sql = "SELECT uid, title, kind, path FROM item WHERE stack_path IS ?1 AND kind != 'stack' ORDER BY path";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([folder], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, kind, path) = row.map_err(sql_err)?;
        let uid = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        let kind = studyfrog_core::plan::item_kind_from_name(&kind)
            .ok_or_else(|| StoreError::Corrupt(format!("kind: {kind:?}")))?;
        out.push(ItemRow {
            uid,
            title,
            kind,
            path,
        });
    }
    Ok(out)
}
