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
        "SELECT COUNT(*) FROM review WHERE at_ms >= ?1 AND at_ms < ?2",
        [start_ms, end_ms],
        |r| r.get(0),
    )
    .map_err(sql_err)
}
