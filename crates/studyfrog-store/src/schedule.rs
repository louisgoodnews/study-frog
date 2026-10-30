//! Persistence of reviews and FSRS memory state.

use std::collections::HashMap;

use rusqlite::{Connection, params};
use studyfrog_core::memory::{MemState, Sm2State};
use studyfrog_core::review::{Review, rating_from_value, rating_value};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

/// FSRS state with the times the scheduler needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredMem {
    /// Stability and difficulty.
    pub state: MemState,
    /// When the item is next due.
    pub due: UnixMs,
    /// When it was last reviewed.
    pub last_review: UnixMs,
}

type MemRow = (f32, f32, i64, i64);

fn mem_of((stability, difficulty, due, last): MemRow) -> StoredMem {
    StoredMem {
        state: MemState {
            stability,
            difficulty,
        },
        due: UnixMs(due),
        last_review: UnixMs(last),
    }
}

/// The memory state of one item, if it has been scheduled.
pub fn load_mem_state(conn: &Connection, item: Uid) -> Result<Option<StoredMem>, StoreError> {
    let sql =
        "SELECT stability, difficulty, due_ms, last_review_ms FROM mem_state WHERE item_uid = ?1";
    let found = conn.query_row(sql, [format_ulid(item)], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    });
    match found {
        Ok(row) => Ok(Some(mem_of(row))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

/// Insert or replace the memory state of an item.
pub fn save_mem_state(conn: &Connection, item: Uid, m: &StoredMem) -> Result<(), StoreError> {
    conn.prepare_cached(
        "INSERT INTO mem_state (item_uid, stability, difficulty, due_ms, last_review_ms) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(item_uid) DO UPDATE SET stability = ?2, difficulty = ?3, due_ms = ?4, last_review_ms = ?5",
    )
    .and_then(|mut s| s.execute(params![format_ulid(item), m.state.stability, m.state.difficulty, m.due.0, m.last_review.0]))
    .map(drop)
    .map_err(sql_err)
}

/// Every scheduled item, for building queues.
pub fn load_all_mem_states(conn: &Connection) -> Result<HashMap<Uid, StoredMem>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT item_uid, stability, difficulty, due_ms, last_review_ms FROM mem_state")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                (r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?),
            ))
        })
        .map_err(sql_err)?;
    let mut all = HashMap::new();
    for row in rows {
        let (id, mem) = row.map_err(sql_err)?;
        let uid = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        all.insert(uid, mem_of(mem));
    }
    Ok(all)
}

/// Append a review to the history.
pub fn insert_review(conn: &Connection, r: &Review) -> Result<(), StoreError> {
    conn.prepare_cached(
        "INSERT INTO review (item_uid, rating, at_ms, elapsed_days) VALUES (?1, ?2, ?3, ?4)",
    )
    .and_then(|mut s| {
        s.execute(params![
            format_ulid(r.item),
            rating_value(r.rating),
            r.at.0,
            r.elapsed_days
        ])
    })
    .map(drop)
    .map_err(sql_err)
}

/// The review history of an item, oldest first.
pub fn load_reviews(conn: &Connection, item: Uid) -> Result<Vec<Review>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT rating, at_ms, elapsed_days FROM review WHERE item_uid = ?1 ORDER BY at_ms, id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(item)], |r| {
            Ok((r.get::<_, u8>(0)?, r.get::<_, i64>(1)?, r.get::<_, f32>(2)?))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (rating, at, elapsed_days) = row.map_err(sql_err)?;
        let rating = rating_from_value(rating)
            .ok_or_else(|| StoreError::Corrupt(format!("rating: {rating}")))?;
        out.push(Review {
            item,
            rating,
            at: UnixMs(at),
            elapsed_days,
        });
    }
    Ok(out)
}

/// SM-2 state with the times the scheduler needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredSm2 {
    /// Ease, repetitions and interval.
    pub state: Sm2State,
    /// When the item is next due.
    pub due: UnixMs,
    /// When it was last reviewed.
    pub last_review: UnixMs,
}

/// The SM-2 state of one item, if it has been scheduled with SM-2.
pub fn load_sm2_state(conn: &Connection, item: Uid) -> Result<Option<StoredSm2>, StoreError> {
    let sql = "SELECT ease, repetitions, interval_days, due_ms, last_review_ms FROM sm2_state WHERE item_uid = ?1";
    let found = conn.query_row(sql, [format_ulid(item)], sm2_row);
    match found {
        Ok(stored) => Ok(Some(stored)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

fn sm2_row(r: &rusqlite::Row) -> rusqlite::Result<StoredSm2> {
    Ok(StoredSm2 {
        state: Sm2State {
            ease: r.get(0)?,
            repetitions: r.get(1)?,
            interval_days: r.get(2)?,
        },
        due: UnixMs(r.get(3)?),
        last_review: UnixMs(r.get(4)?),
    })
}

/// Insert or replace the SM-2 state of an item.
pub fn save_sm2_state(conn: &Connection, item: Uid, m: &StoredSm2) -> Result<(), StoreError> {
    conn.prepare_cached(
        "INSERT INTO sm2_state (item_uid, ease, repetitions, interval_days, due_ms, last_review_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(item_uid) DO UPDATE SET ease = ?2, repetitions = ?3, interval_days = ?4, due_ms = ?5,
             last_review_ms = ?6",
    )
    .and_then(|mut s| {
        s.execute(params![format_ulid(item), m.state.ease, m.state.repetitions, m.state.interval_days, m.due.0, m.last_review.0])
    })
    .map(drop)
    .map_err(sql_err)
}

/// What the queue builder needs to know about a scheduled item, whatever scheduled it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scheduled {
    /// FSRS state, when the item has one (used by difficulty filters and orders).
    pub memory: Option<MemState>,
    /// When the item is next due, according to the item's current scheduler.
    pub due: UnixMs,
    /// When it was last reviewed.
    pub last_review: UnixMs,
}

fn all_sm2(conn: &Connection) -> Result<HashMap<Uid, StoredSm2>, StoreError> {
    let sql =
        "SELECT ease, repetitions, interval_days, due_ms, last_review_ms, item_uid FROM sm2_state";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((sm2_row(r)?, r.get::<_, String>(5)?)))
        .map_err(sql_err)?;
    let mut all = HashMap::new();
    for row in rows {
        let (stored, id) = row.map_err(sql_err)?;
        let uid = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        all.insert(uid, stored);
    }
    Ok(all)
}

/// Every scheduled item. `scheduler_of` says which algorithm an item uses now; when an item only has
/// state from the other algorithm (its stack was switched), that state's due date keeps it scheduled
/// until its next review creates fresh state in the new algorithm.
pub fn load_schedules(
    conn: &Connection,
    scheduler_of: impl Fn(Uid) -> Scheduler,
) -> Result<HashMap<Uid, Scheduled>, StoreError> {
    let (fsrs, sm2) = (load_all_mem_states(conn)?, all_sm2(conn)?);
    let uids: std::collections::HashSet<Uid> = fsrs.keys().chain(sm2.keys()).copied().collect();
    let pick = |uid: Uid| {
        let (f, s) = (fsrs.get(&uid), sm2.get(&uid));
        let (due, last) = match scheduler_of(uid) {
            Scheduler::Fsrs => f
                .map(|m| (m.due, m.last_review))
                .or(s.map(|m| (m.due, m.last_review))),
            Scheduler::Sm2 => s
                .map(|m| (m.due, m.last_review))
                .or(f.map(|m| (m.due, m.last_review))),
        }?;
        Some(Scheduled {
            memory: f.map(|m| m.state),
            due,
            last_review: last,
        })
    };
    Ok(uids
        .into_iter()
        .filter_map(|uid| pick(uid).map(|s| (uid, s)))
        .collect())
}
