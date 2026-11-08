//! Focus blocks of the focus timer (STORY-0112): written when a work block ends, read per calendar
//! session occurrence and per time span.

use rusqlite::{Connection, OptionalExtension, params};
use studyfrog_core::civil::{Date, format_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};

/// One finished work block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusBlock {
    /// When the block started (Unix milliseconds).
    pub started_ms: i64,
    /// When it ended.
    pub ended_ms: i64,
    /// Minutes of focus counted (pauses excluded).
    pub minutes: u32,
    /// The stack folder it was for, if one was chosen.
    pub stack: Option<String>,
    /// The calendar session occurrence it counts towards: the session and its local date.
    pub session: Option<(Uid, Date)>,
}

/// Store a block.
pub fn insert_focus_block(conn: &Connection, b: &FocusBlock) -> Result<(), StoreError> {
    let (uid, date) = match b.session {
        Some((u, d)) => (Some(format_ulid(u)), Some(format_date(d))),
        None => (None, None),
    };
    conn.execute(
        "INSERT INTO focus_block (started_ms, ended_ms, minutes, stack, session_uid, session_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![b.started_ms, b.ended_ms, b.minutes, b.stack, uid, date],
    )
    .map(|_| ())
    .map_err(sql_err)
}

/// Minutes of focus recorded against one occurrence of a calendar session.
pub fn focus_minutes_for(conn: &Connection, session: Uid, date: Date) -> Result<u32, StoreError> {
    conn.query_row(
        "SELECT SUM(minutes) FROM focus_block WHERE session_uid = ?1 AND session_date = ?2",
        params![format_ulid(session), format_date(date)],
        |r| r.get::<_, Option<u32>>(0),
    )
    .optional()
    .map(|m| m.flatten().unwrap_or(0))
    .map_err(sql_err)
}

/// Minutes of focus of the blocks that ended in `[from_ms, to_ms)`.
pub fn focus_minutes_between(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
) -> Result<u32, StoreError> {
    conn.query_row(
        "SELECT SUM(minutes) FROM focus_block WHERE ended_ms >= ?1 AND ended_ms < ?2",
        params![from_ms, to_ms],
        |r| r.get::<_, Option<u32>>(0),
    )
    .map(|m| m.unwrap_or(0))
    .map_err(sql_err)
}
