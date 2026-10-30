//! Loading runs, their queues, filters and event logs.

use rusqlite::Connection;
use studyfrog_core::event::{RunEvent, event_from_parts};
use studyfrog_core::plan::{
    DifficultyRange, ItemFilter, RunPlan, effect_from_name, item_kind_from_name, order_from_name,
    state_filter_from_name,
};
use studyfrog_core::run::{Run, run_mode_from_name, run_state_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};
use crate::runs::{corrupt, uid_of};

/// Raw columns of a `run` row.
struct RunRow {
    mode: String,
    state: String,
    started_at: Option<i64>,
    order: String,
    effect: String,
    seed: i64,
    limit: Option<u32>,
    requeue: bool,
    filter_state: String,
    diff: (Option<f32>, Option<f32>),
    shuffle: bool,
}

fn read_run_row(conn: &Connection, uid: Uid) -> Result<Option<RunRow>, StoreError> {
    let sql =
        "SELECT mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again,
                      filter_state, filter_diff_min, filter_diff_max, shuffle FROM run WHERE uid = ?1";
    let found = conn.query_row(sql, [format_ulid(uid)], |r| {
        Ok(RunRow {
            mode: r.get(0)?,
            state: r.get(1)?,
            started_at: r.get(2)?,
            order: r.get(3)?,
            effect: r.get(4)?,
            seed: r.get(5)?,
            limit: r.get(6)?,
            requeue: r.get(7)?,
            filter_state: r.get(8)?,
            diff: (r.get(9)?, r.get(10)?),
            shuffle: r.get(11)?,
        })
    });
    match found {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

/// A run with its queue and full plan.
pub fn load_run(conn: &Connection, uid: Uid) -> Result<Option<Run>, StoreError> {
    let Some(row) = read_run_row(conn, uid)? else {
        return Ok(None);
    };
    let difficulty = match row.diff {
        (Some(min), Some(max)) => Some(DifficultyRange { min, max }),
        _ => None,
    };
    let filter = ItemFilter {
        kinds: load_kinds(conn, uid)?,
        stacks: load_stacks(conn, uid)?,
        tags_any: load_tags(conn, uid)?,
        state: state_filter_from_name(&row.filter_state)
            .ok_or_else(|| corrupt("filter_state", &row.filter_state))?,
        difficulty,
    };
    let plan = RunPlan {
        order: order_from_name(&row.order).ok_or_else(|| corrupt("order", &row.order))?,
        filter,
        effect: effect_from_name(&row.effect).ok_or_else(|| corrupt("effect", &row.effect))?,
        shuffle: row.shuffle,
        seed: row.seed as u64,
        limit: row.limit,
        requeue_again: row.requeue,
    };
    Ok(Some(Run {
        uid,
        mode: run_mode_from_name(&row.mode).ok_or_else(|| corrupt("mode", &row.mode))?,
        plan,
        queue: load_queue(conn, uid)?,
        state: run_state_from_name(&row.state).ok_or_else(|| corrupt("state", &row.state))?,
        started_at: row.started_at.map(UnixMs),
    }))
}

/// Ordered text column of a `run_filter_*` table.
fn load_list(
    conn: &Connection,
    table: &str,
    column: &str,
    run: Uid,
) -> Result<Vec<String>, StoreError> {
    let sql = format!("SELECT {column} FROM {table} WHERE run_uid = ?1 ORDER BY pos");
    let mut stmt = conn.prepare(&sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(run)], |r| r.get(0))
        .map_err(sql_err)?;
    rows.collect::<Result<_, _>>().map_err(sql_err)
}

fn load_kinds(
    conn: &Connection,
    run: Uid,
) -> Result<Vec<studyfrog_core::plan::ItemKind>, StoreError> {
    let names = load_list(conn, "run_filter_kind", "kind", run)?;
    names
        .iter()
        .map(|n| item_kind_from_name(n).ok_or_else(|| corrupt("kind", n)))
        .collect()
}

fn load_stacks(conn: &Connection, run: Uid) -> Result<Vec<Uid>, StoreError> {
    load_list(conn, "run_filter_stack", "stack_uid", run)?
        .iter()
        .map(|t| uid_of(t))
        .collect()
}

fn load_tags(conn: &Connection, run: Uid) -> Result<Vec<String>, StoreError> {
    load_list(conn, "run_filter_tag", "tag", run)
}

fn load_queue(conn: &Connection, run: Uid) -> Result<Vec<Uid>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT item_uid FROM run_queue WHERE run_uid = ?1 ORDER BY pos")
        .map_err(sql_err)?;
    let ids: Vec<String> = stmt
        .query_map([format_ulid(run)], |r| r.get(0))
        .map_err(sql_err)?
        .collect::<Result<_, _>>()
        .map_err(sql_err)?;
    ids.iter().map(|t| uid_of(t)).collect()
}

/// The event log of a run in sequence order.
pub fn load_events(conn: &Connection, run: Uid) -> Result<Vec<RunEvent>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT seq, t_ms, item_uid, kind, payload FROM run_event WHERE run_uid = ?1 ORDER BY seq")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(run)], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut events = Vec::new();
    for row in rows {
        let (seq, t_ms, item, kind, payload) = row.map_err(sql_err)?;
        let kind =
            event_from_parts(&kind, payload.as_deref()).ok_or_else(|| corrupt("event", &kind))?;
        let item = item.as_deref().map(uid_of).transpose()?;
        events.push(RunEvent {
            run,
            seq,
            t_ms,
            item,
            kind,
        });
    }
    Ok(events)
}
