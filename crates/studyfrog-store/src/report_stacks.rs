//! Per-stack numbers for the CSV export of the reports (STORY-0098): items, due, learned, mastered,
//! average interval and retention of every stack, counting the items whose primary stack it is
//! (the same rule as the per-stack achievements).

use rusqlite::Connection;
use studyfrog_core::achievement::MASTERED_DAYS;

use crate::error::{StoreError, sql_err};

/// One stack's numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct StackReportRow {
    /// The stack's folder in the vault.
    pub folder: String,
    /// Its title (the folder when it has none).
    pub title: String,
    /// Notes, cards and questions in it.
    pub items: u32,
    /// Items due at the given time.
    pub due: u32,
    /// Items with at least one review.
    pub learned: u32,
    /// Items scheduled at least `MASTERED_DAYS` ahead (FSRS stability or SM-2 interval).
    pub mastered: u32,
    /// The mean current interval in days of the scheduled items, if any are scheduled.
    pub average_interval_days: Option<f64>,
    /// Reviews of its items.
    pub reviews: u32,
    /// Of those, the ones not rated Again.
    pub recalled: u32,
}

const SQL: &str = "
SELECT s.folder, COALESCE(t.title, s.folder),
 (SELECT COUNT(*) FROM item i WHERE i.stack_path = s.folder AND i.kind != 'stack'),
 (SELECT COUNT(*) FROM item i
    LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary q ON q.item_uid = i.uid
    WHERE i.stack_path = s.folder AND i.kind != 'stack' AND COALESCE(m.due_ms, q.due_ms) <= ?1),
 (SELECT COUNT(DISTINCT r.item_uid) FROM review_once r JOIN item i ON i.uid = r.item_uid
    WHERE i.stack_path = s.folder),
 (SELECT COUNT(*) FROM item i
    LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary q ON q.item_uid = i.uid
    WHERE i.stack_path = s.folder AND i.kind != 'stack'
      AND (m.stability >= ?2 OR q.interval_days >= ?2)),
 (SELECT AVG(COALESCE((m.due_ms - m.last_review_ms) / 86400000.0, q.interval_days)) FROM item i
    LEFT JOIN mem_primary m ON m.item_uid = i.uid LEFT JOIN sm2_primary q ON q.item_uid = i.uid
    WHERE i.stack_path = s.folder AND i.kind != 'stack'),
 (SELECT COUNT(*) FROM review_once r JOIN item i ON i.uid = r.item_uid WHERE i.stack_path = s.folder),
 (SELECT COUNT(*) FROM review_once r JOIN item i ON i.uid = r.item_uid
    WHERE i.stack_path = s.folder AND r.rating > 1)
FROM stack_identity s LEFT JOIN item t ON t.uid = s.uid
ORDER BY s.folder";

fn count(v: i64) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// Every stack's numbers at `now_ms`, ordered by folder.
pub fn load_stack_report(
    conn: &Connection,
    now_ms: i64,
) -> Result<Vec<StackReportRow>, StoreError> {
    let mut stmt = conn.prepare(SQL).map_err(sql_err)?;
    let rows = stmt
        .query_map((now_ms, f64::from(MASTERED_DAYS)), |r| {
            Ok(StackReportRow {
                folder: r.get(0)?,
                title: r.get(1)?,
                items: count(r.get(2)?),
                due: count(r.get(3)?),
                learned: count(r.get(4)?),
                mastered: count(r.get(5)?),
                average_interval_days: r.get(6)?,
                reviews: count(r.get(7)?),
                recalled: count(r.get(8)?),
            })
        })
        .map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}

/// Items with at least one review, in the whole vault.
pub fn count_learned(conn: &Connection) -> Result<u32, StoreError> {
    conn.query_row(
        "SELECT COUNT(DISTINCT item_uid) FROM review_once",
        [],
        |r| r.get::<_, i64>(0),
    )
    .map(count)
    .map_err(sql_err)
}
