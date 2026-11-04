//! What achievements are measured on, read from the history, and which ones were unlocked.

use rusqlite::{Connection, params};
use studyfrog_core::achievement::MASTERED_DAYS;
use studyfrog_core::review::{Rating, rating_from_value};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};

/// One stored review with the key of its item's stack (its uid), if the item is in a stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    /// The item.
    pub item: Uid,
    /// The stack's uid text.
    pub stack: Option<String>,
    /// The rating given.
    pub rating: Rating,
    /// When, in Unix milliseconds.
    pub at_ms: i64,
}

fn uid_of(text: &str) -> Result<Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}")))
}

/// Every review, oldest first, with its stack.
pub fn load_review_rows(conn: &Connection) -> Result<Vec<ReviewRow>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT r.item_uid, s.uid, r.rating, r.at_ms FROM review_once r \
             LEFT JOIN item i ON i.uid = r.item_uid \
             LEFT JOIN stack_identity s ON s.folder = i.stack_path \
             ORDER BY r.at_ms, r.id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, u8>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (item, stack, rating, at_ms) = row.map_err(sql_err)?;
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        out.push(ReviewRow {
            item: uid_of(&item)?,
            stack,
            rating,
            at_ms,
        });
    }
    Ok(out)
}

/// An item and the key of its stack.
pub type StackedItem = (Uid, Option<String>);

/// Items scheduled at least [`MASTERED_DAYS`] ahead of their last review, with their stack. FSRS items
/// by stability, SM-2 items by interval.
pub fn load_mastered(conn: &Connection) -> Result<Vec<StackedItem>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT m.item_uid, s.uid FROM (\
               SELECT item_uid FROM mem_state WHERE stability >= ?1 \
               UNION SELECT item_uid FROM sm2_state WHERE interval_days >= ?1) m \
             LEFT JOIN item i ON i.uid = m.item_uid \
             LEFT JOIN stack_identity s ON s.folder = i.stack_path",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([f64::from(MASTERED_DAYS)], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (item, stack) = row.map_err(sql_err)?;
        out.push((uid_of(&item)?, stack));
    }
    Ok(out)
}

/// Runs that finished.
pub fn count_finished_runs(conn: &Connection) -> Result<u32, StoreError> {
    conn.query_row(
        "SELECT COUNT(*) FROM run WHERE state = 'finished'",
        [],
        |r| r.get(0),
    )
    .map_err(sql_err)
}

/// Remembered unlocks as (rule id, scope key, unlocked at in milliseconds).
pub fn load_unlocks(conn: &Connection) -> Result<Vec<(String, String, i64)>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT rule_id, scope_key, unlocked_ms FROM achievement_unlock ORDER BY unlocked_ms",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}

/// Remember an unlock; an existing one keeps its original time.
pub fn save_unlock(conn: &Connection, rule: &str, key: &str, at_ms: i64) -> Result<(), StoreError> {
    conn.execute(
        "INSERT OR IGNORE INTO achievement_unlock (rule_id, scope_key, unlocked_ms) VALUES (?1, ?2, ?3)",
        params![rule, key, at_ms],
    )
    .map(drop)
    .map_err(sql_err)
}
