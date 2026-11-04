//! What the home screen reads besides the schedule: recent runs and the day's review count.

use rusqlite::Connection;
use studyfrog_core::uid::Uid;

use crate::error::{StoreError, sql_err};
use crate::runs::uid_of;

/// A run that was started, with how it went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRow {
    /// The run.
    pub uid: Uid,
    /// `flip` or `answer`.
    pub mode: String,
    /// `finished`, `aborted`, `running`, ...
    pub state: String,
    /// When it started, in Unix milliseconds.
    pub started_ms: i64,
    /// Items rated.
    pub rated: u32,
    /// Items skipped.
    pub skipped: u32,
}

/// The runs started most recently, newest first.
pub fn recent_runs(conn: &Connection, limit: usize) -> Result<Vec<RunRow>, StoreError> {
    let sql = "SELECT r.uid, r.mode, r.state, r.started_at_ms,
                      (SELECT COUNT(*) FROM run_event e WHERE e.run_uid = r.uid AND e.kind = 'item_rated'),
                      (SELECT COUNT(*) FROM run_event e WHERE e.run_uid = r.uid AND e.kind = 'item_skipped')
               FROM run r WHERE r.started_at_ms IS NOT NULL ORDER BY r.started_at_ms DESC LIMIT ?1";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (uid, mode, state, started_ms, rated, skipped) = row.map_err(sql_err)?;
        out.push(RunRow {
            uid: uid_of(&uid)?,
            mode,
            state,
            started_ms,
            rated,
            skipped,
        });
    }
    Ok(out)
}

/// Reviews recorded from `start_ms` (inclusive) to `end_ms` (exclusive).
pub fn count_reviews_between(
    conn: &Connection,
    start_ms: i64,
    end_ms: i64,
) -> Result<u32, StoreError> {
    conn.query_row(
        "SELECT COUNT(*) FROM review_once WHERE at_ms >= ?1 AND at_ms < ?2",
        [start_ms, end_ms],
        |r| r.get(0),
    )
    .map_err(sql_err)
}

/// What the report explorer needs to know about an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexRow {
    /// The item.
    pub uid: Uid,
    /// Title.
    pub title: String,
    /// Note, card or question.
    pub kind: studyfrog_core::plan::ItemKind,
    /// The folder of the stack it is in; `None` for the inbox.
    pub stack: Option<String>,
    /// The folders of the other stacks that reference it.
    pub also_in: Vec<String>,
}

/// Every item except stacks, in path order.
pub fn item_index(conn: &Connection) -> Result<Vec<IndexRow>, StoreError> {
    let sql = "SELECT uid, title, kind, stack_path FROM item WHERE kind != 'stack' ORDER BY path";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut also = crate::refs_index::referencing_folders(conn)?;
    let mut out = Vec::new();
    for row in rows {
        let (id, title, kind, stack) = row.map_err(sql_err)?;
        let uid = studyfrog_md::ulid::parse_ulid(&id)
            .ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        let kind = studyfrog_core::plan::item_kind_from_name(&kind)
            .ok_or_else(|| StoreError::Corrupt(format!("kind: {kind:?}")))?;
        out.push(IndexRow {
            uid,
            title,
            kind,
            also_in: also.remove(&id).unwrap_or_default(),
            stack,
        });
    }
    Ok(out)
}

/// Whether the row belongs to the stack at `folder` or inside it, through its own folder or a reference.
pub fn row_in_stack(i: &IndexRow, folder: &str) -> bool {
    let inside = |f: &str| f == folder || f.starts_with(&format!("{folder}/"));
    i.stack.as_deref().is_some_and(inside) || i.also_in.iter().any(|f| inside(f))
}
