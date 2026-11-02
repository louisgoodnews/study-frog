//! Read-only queries over stack settings and today's activity.

use std::collections::HashMap;

use rusqlite::{Connection, params};
use studyfrog_core::scheduler::{Scheduler, scheduler_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};

/// The stack an item belongs to and that stack's scheduling settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemHome {
    /// The owning stack.
    pub stack: Uid,
    /// The stack's scheduler, `None` for the vault default.
    pub scheduler: Option<Scheduler>,
    /// The stack's retention override.
    pub request_retention: Option<f32>,
    /// The stack's switch for familiarity, `None` for the vault default.
    pub familiarity: Option<bool>,
}

/// Daily allowances of a stack; `None` means unlimited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackLimits {
    /// New items per day.
    pub new_items: Option<u32>,
    /// Reviews per day.
    pub reviews: Option<u32>,
}

/// What a stack has already done within a day window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DailyCounts {
    /// Distinct items whose very first review happened in the window.
    pub new_done: u32,
    /// Distinct items reviewed in the window that had been reviewed before it.
    pub reviews_done: u32,
}

fn uid_of(text: &str) -> Result<Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}")))
}

fn scheduler_of(name: Option<String>) -> Result<Option<Scheduler>, StoreError> {
    name.map(|n| {
        scheduler_from_name(&n).ok_or_else(|| StoreError::Corrupt(format!("scheduler: {n:?}")))
    })
    .transpose()
}

/// Home of every indexed item that sits in a stack (items in the inbox are absent).
pub fn item_homes(conn: &Connection) -> Result<HashMap<Uid, ItemHome>, StoreError> {
    let sql = "SELECT i.uid, s.uid, s.scheduler, s.request_retention, s.familiarity
               FROM item i JOIN stack_setting s ON s.folder = i.stack_path";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<f32>>(3)?,
                r.get::<_, Option<bool>>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut homes = HashMap::new();
    for row in rows {
        let (item, stack, scheduler, retention, familiarity) = row.map_err(sql_err)?;
        homes.insert(
            uid_of(&item)?,
            ItemHome {
                stack: uid_of(&stack)?,
                scheduler: scheduler_of(scheduler)?,
                request_retention: retention,
                familiarity,
            },
        );
    }
    Ok(homes)
}

/// Daily limits of every stack that sets any.
pub fn stack_limits(conn: &Connection) -> Result<HashMap<Uid, StackLimits>, StoreError> {
    let sql = "SELECT uid, daily_new_limit, daily_review_limit FROM stack_setting
               WHERE daily_new_limit IS NOT NULL OR daily_review_limit IS NOT NULL";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<u32>>(1)?,
                r.get::<_, Option<u32>>(2)?,
            ))
        })
        .map_err(sql_err)?;
    let mut limits = HashMap::new();
    for row in rows {
        let (stack, new_items, reviews) = row.map_err(sql_err)?;
        limits.insert(uid_of(&stack)?, StackLimits { new_items, reviews });
    }
    Ok(limits)
}

const COUNTS_SQL: &str = "
WITH first AS (SELECT item_uid, MIN(at_ms) AS first_at FROM review GROUP BY item_uid)
SELECT s.uid,
       COUNT(DISTINCT CASE WHEN f.first_at >= ?1 THEN r.item_uid END),
       COUNT(DISTINCT CASE WHEN f.first_at < ?1 THEN r.item_uid END)
FROM review r
JOIN first f ON f.item_uid = r.item_uid
JOIN item i ON i.uid = r.item_uid
JOIN stack_setting s ON s.folder = i.stack_path
WHERE r.at_ms >= ?1 AND r.at_ms < ?2
GROUP BY s.uid";

/// Per-stack activity within `[start, end)`, from the review history. Practice-only runs write no
/// reviews and so never count.
pub fn daily_counts(
    conn: &Connection,
    start: UnixMs,
    end: UnixMs,
) -> Result<HashMap<Uid, DailyCounts>, StoreError> {
    let mut stmt = conn.prepare(COUNTS_SQL).map_err(sql_err)?;
    let rows = stmt
        .query_map(params![start.0, end.0], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })
        .map_err(sql_err)?;
    let mut counts = HashMap::new();
    for row in rows {
        let (stack, new_done, reviews_done) = row.map_err(sql_err)?;
        counts.insert(
            uid_of(&stack)?,
            DailyCounts {
                new_done,
                reviews_done,
            },
        );
    }
    Ok(counts)
}

/// Home of one item, if it sits in a stack. A single indexed lookup, cheap enough per rating.
pub fn item_home(conn: &Connection, item: Uid) -> Result<Option<ItemHome>, StoreError> {
    let sql = "SELECT s.uid, s.scheduler, s.request_retention, s.familiarity
               FROM item i JOIN stack_setting s ON s.folder = i.stack_path WHERE i.uid = ?1";
    let found = conn.query_row(sql, [studyfrog_md::ulid::format_ulid(item)], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<f32>>(2)?,
            r.get::<_, Option<bool>>(3)?,
        ))
    });
    match found {
        Ok((stack, scheduler, request_retention, familiarity)) => Ok(Some(ItemHome {
            stack: uid_of(&stack)?,
            scheduler: scheduler_of(scheduler)?,
            request_retention,
            familiarity,
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}
