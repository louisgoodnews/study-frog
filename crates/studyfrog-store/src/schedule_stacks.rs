//! The schedule of every (item, stack) pair, for queues and planners that look at one stack at a time (ADR 0011).

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::memory::MemState;
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::parse_ulid;

use crate::error::{StoreError, sql_err};
use crate::schedule::Scheduled;

/// What FSRS and SM-2 each know about one (item, stack).
type Both = (Option<Scheduled>, Option<Scheduled>);

type Row = (String, String, f32, f32, i64, i64);

fn uid_of(text: &str) -> Result<Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}")))
}

fn read(conn: &Connection, sql: &str) -> Result<Vec<Row>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })
        .map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}

/// Every scheduled (item, stack). `scheduler_of` names the algorithm a stack uses now; when a pair only has state
/// from the other algorithm (its stack was switched), that state's due date keeps it scheduled until its next review.
pub fn load_stack_schedules(
    conn: &Connection,
    scheduler_of: impl Fn(Uid) -> Scheduler,
) -> Result<HashMap<(Uid, Uid), Scheduled>, StoreError> {
    let fsrs = read(
        conn,
        "SELECT item_uid, stack_uid, stability, difficulty, due_ms, last_review_ms FROM mem_state",
    )?;
    let sm2 = read(
        conn,
        "SELECT item_uid, stack_uid, 0.0, 0.0, due_ms, last_review_ms FROM sm2_state",
    )?;
    let mut all: HashMap<(Uid, Uid), Both> = HashMap::new();
    for (is_fsrs, rows) in [(true, fsrs), (false, sm2)] {
        for (item, stack, stability, difficulty, due, last) in rows {
            let entry = Scheduled {
                memory: is_fsrs.then_some(MemState {
                    stability,
                    difficulty,
                }),
                due: UnixMs(due),
                last_review: UnixMs(last),
            };
            let slot = all.entry((uid_of(&item)?, uid_of(&stack)?)).or_default();
            if is_fsrs {
                slot.0 = Some(entry);
            } else {
                slot.1 = Some(entry);
            }
        }
    }
    Ok(all
        .into_iter()
        .filter_map(|(key @ (_, stack), (f, s))| {
            let pick = match scheduler_of(stack) {
                Scheduler::Fsrs => f.or(s),
                Scheduler::Sm2 => s.or(f),
            };
            let memory = f.and_then(|m| m.memory);
            pick.map(|p| (key, Scheduled { memory, ..p }))
        })
        .collect())
}
