//! The answer timings of each item's first review: what the starting difficulty is learned from
//! (STORY-0177). Read-only.

use rusqlite::Connection;

use crate::error::{StoreError, sql_err};

/// The most recent first-review timings read.
pub const MAX_FIRST_TIMINGS: i64 = 500;
/// The newest reviews looked at, so a history without timings costs no more than this.
pub const MAX_SCANNED: i64 = 5_000;

/// The first-review timings of the reviews made before the instant `before_ms`: (answer ms, expected ms),
/// newest first, found among the newest [`MAX_SCANNED`] of those reviews. A replay asks the same question
/// for the time of each review, so it gets the answer that was current when the review was made.
pub fn first_review_timings_before(
    conn: &Connection,
    before_ms: i64,
) -> Result<Vec<(u32, u32)>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT r.answer_ms, r.expected_ms FROM \
             (SELECT * FROM review_once WHERE at_ms < ?3 ORDER BY id DESC LIMIT ?2) r \
             WHERE r.answer_ms IS NOT NULL AND r.expected_ms IS NOT NULL \
             AND NOT EXISTS (SELECT 1 FROM review e WHERE e.item_uid = r.item_uid AND e.at_ms < r.at_ms) \
             ORDER BY r.id DESC LIMIT ?1",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map(
            rusqlite::params![MAX_FIRST_TIMINGS, MAX_SCANNED, before_ms],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(sql_err)?;
    rows.collect::<Result<Vec<(u32, u32)>, _>>()
        .map_err(sql_err)
}

/// The first-review timings of every review (see [`first_review_timings_before`]).
pub fn first_review_timings(conn: &Connection) -> Result<Vec<(u32, u32)>, StoreError> {
    first_review_timings_before(conn, i64::MAX)
}
