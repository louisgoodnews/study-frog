//! The history the reports are computed from, read in bulk.

use rusqlite::Connection;
use studyfrog_core::event::{RunEvent, event_from_parts};
use studyfrog_core::review::{Rating, rating_from_value};
use studyfrog_core::uid::Uid;

use crate::error::{StoreError, sql_err};
use crate::runs::{corrupt, uid_of};

/// One stored review.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryRow {
    /// The item.
    pub item: Uid,
    /// The rating given.
    pub rating: Rating,
    /// When, in Unix milliseconds.
    pub at_ms: i64,
    /// Days since the item's previous review (0 for its first).
    pub elapsed_days: f32,
}

/// Every review, oldest first.
pub fn load_review_history(conn: &Connection) -> Result<Vec<HistoryRow>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT item_uid, rating, at_ms, elapsed_days FROM review ORDER BY at_ms, id")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u8>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, f32>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (item, rating, at_ms, elapsed_days) = row.map_err(sql_err)?;
        let rating =
            rating_from_value(rating).ok_or_else(|| corrupt("rating", &rating.to_string()))?;
        out.push(HistoryRow {
            item: uid_of(&item)?,
            rating,
            at_ms,
            elapsed_days,
        });
    }
    Ok(out)
}

/// The event logs of all runs, grouped by run and in sequence order.
pub fn load_all_events(conn: &Connection) -> Result<Vec<RunEvent>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT run_uid, seq, t_ms, item_uid, kind, payload FROM run_event ORDER BY run_uid, seq")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(sql_err)?;
    let mut events = Vec::new();
    for row in rows {
        let (run, seq, t_ms, item, kind, payload) = row.map_err(sql_err)?;
        let kind =
            event_from_parts(&kind, payload.as_deref()).ok_or_else(|| corrupt("event", &kind))?;
        let item = item.as_deref().map(uid_of).transpose()?;
        events.push(RunEvent {
            run: uid_of(&run)?,
            seq,
            t_ms,
            item,
            kind,
        });
    }
    Ok(events)
}

/// Every scheduled item as (due time in milliseconds, current interval in days): FSRS items with
/// their stability as the interval, SM-2 items with their interval.
pub fn load_scheduled(conn: &Connection) -> Result<Vec<(i64, f32)>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT due_ms, stability FROM mem_state \
             UNION ALL SELECT due_ms, interval_days FROM sm2_state",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, f32>(1)?)))
        .map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}
