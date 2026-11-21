//! The answer timings of each item's first review: what the starting difficulty is learned from
//! (STORY-0177). Read-only.

use rusqlite::Connection;

use crate::error::{StoreError, sql_err};

/// The most recent first-review timings read.
pub const MAX_FIRST_TIMINGS: i64 = 500;

/// (answer ms, expected ms) of the first review of each item that has both, newest first.
pub fn first_review_timings(conn: &Connection) -> Result<Vec<(u32, u32)>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT r.answer_ms, r.expected_ms FROM review_once r \
             JOIN (SELECT item_uid, MIN(at_ms) AS first_at FROM review_once GROUP BY item_uid) f \
             ON r.item_uid = f.item_uid AND r.at_ms = f.first_at \
             WHERE r.answer_ms IS NOT NULL AND r.expected_ms IS NOT NULL \
             ORDER BY r.at_ms DESC LIMIT ?1",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([MAX_FIRST_TIMINGS], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<(u32, u32)>, _>>()
        .map_err(sql_err)
}
