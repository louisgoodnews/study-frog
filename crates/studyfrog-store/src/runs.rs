//! Persistence of rehearsal runs and their append-only event log.

use rusqlite::{Connection, params};
use studyfrog_core::event::{RunEvent, event_from_parts, event_name, event_payload};
use studyfrog_core::plan::{
    RunPlan, effect_from_name, effect_name, filter_all, order_from_name, order_name,
};
use studyfrog_core::run::{
    Run, RunState, run_mode_from_name, run_mode_name, run_state_from_name, run_state_name,
};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn corrupt(what: &str, value: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {value:?}"))
}

fn uid_of(text: &str) -> Result<Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| corrupt("ulid", text))
}

/// Store a new run with its queue.
pub fn insert_run(conn: &Connection, run: &Run) -> Result<(), StoreError> {
    let p = &run.plan;
    conn.execute(
        "INSERT INTO run (uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            format_ulid(run.uid),
            run_mode_name(run.mode),
            run_state_name(run.state),
            run.started_at.map(|t| t.0),
            order_name(p.order),
            effect_name(p.effect),
            p.seed as i64,
            p.limit,
            p.requeue_again
        ],
    )
    .map_err(sql_err)?;
    let mut stmt = conn
        .prepare_cached("INSERT INTO run_queue (run_uid, pos, item_uid) VALUES (?1, ?2, ?3)")
        .map_err(sql_err)?;
    for (pos, item) in run.queue.iter().enumerate() {
        stmt.execute(params![
            format_ulid(run.uid),
            pos as i64,
            format_ulid(*item)
        ])
        .map_err(sql_err)?;
    }
    Ok(())
}

/// Record a lifecycle change (and the wall-clock start, when it happens).
pub fn set_run_state(
    conn: &Connection,
    run: Uid,
    state: RunState,
    started_at: Option<UnixMs>,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE run SET state = ?2, started_at_ms = COALESCE(?3, started_at_ms) WHERE uid = ?1",
        params![
            format_ulid(run),
            run_state_name(state),
            started_at.map(|t| t.0)
        ],
    )
    .map(drop)
    .map_err(sql_err)
}

/// Append events to the log.
pub fn append_events(conn: &Connection, events: &[RunEvent]) -> Result<(), StoreError> {
    let mut stmt = conn
        .prepare_cached(
            "INSERT INTO run_event (run_uid, seq, t_ms, item_uid, kind, payload) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(sql_err)?;
    for e in events {
        let item = e.item.map(format_ulid);
        stmt.execute(params![
            format_ulid(e.run),
            e.seq,
            e.t_ms,
            item,
            event_name(&e.kind),
            event_payload(&e.kind)
        ])
        .map_err(sql_err)?;
    }
    Ok(())
}

/// A run with its queue and plan. The plan's filter is not stored and comes back empty.
pub fn load_run(conn: &Connection, uid: Uid) -> Result<Option<Run>, StoreError> {
    let sql = "SELECT mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again FROM run WHERE uid = ?1";
    let row = conn
        .query_row(sql, [format_ulid(uid)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, Option<u32>>(6)?,
                r.get::<_, bool>(7)?,
            ))
        })
        .map(Some)
        .or_else(|e| {
            if e == rusqlite::Error::QueryReturnedNoRows {
                Ok(None)
            } else {
                Err(e)
            }
        })
        .map_err(sql_err)?;
    let Some((mode, state, started, order, effect, seed, limit, requeue)) = row else {
        return Ok(None);
    };
    let plan = RunPlan {
        order: order_from_name(&order).ok_or_else(|| corrupt("order", &order))?,
        filter: filter_all(),
        effect: effect_from_name(&effect).ok_or_else(|| corrupt("effect", &effect))?,
        seed: seed as u64,
        limit,
        requeue_again: requeue,
    };
    Ok(Some(Run {
        uid,
        mode: run_mode_from_name(&mode).ok_or_else(|| corrupt("mode", &mode))?,
        plan,
        queue: load_queue(conn, uid)?,
        state: run_state_from_name(&state).ok_or_else(|| corrupt("state", &state))?,
        started_at: started.map(UnixMs),
    }))
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

/// Crash recovery: every run still idle, running or paused becomes aborted and its log is closed
/// with a `run_aborted` event. Returns how many runs were recovered.
pub fn abort_unfinished_runs(conn: &mut Connection) -> Result<usize, StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    let unfinished: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT uid FROM run WHERE state IN ('idle', 'running', 'paused')")
            .map_err(sql_err)?;
        stmt.query_map([], |r| r.get(0))
            .map_err(sql_err)?
            .collect::<Result<_, _>>()
            .map_err(sql_err)?
    };
    for text in &unfinished {
        let run = uid_of(text)?;
        let (next_seq, last_t): (u32, u32) = tx
            .query_row("SELECT COALESCE(MAX(seq) + 1, 0), COALESCE(MAX(t_ms), 0) FROM run_event WHERE run_uid = ?1", [text], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(sql_err)?;
        let closing = RunEvent {
            run,
            seq: next_seq,
            t_ms: last_t,
            item: None,
            kind: studyfrog_core::event::EventKind::RunAborted,
        };
        append_events(&tx, &[closing])?;
        set_run_state(&tx, run, RunState::Aborted, None)?;
    }
    tx.commit().map_err(sql_err)?;
    Ok(unfinished.len())
}
