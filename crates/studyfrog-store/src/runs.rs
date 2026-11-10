//! Persistence of rehearsal runs and their append-only event log.

use rusqlite::{Connection, params};
use studyfrog_core::event::{RunEvent, event_name, event_payload};
use studyfrog_core::pacing::{countdown_secs, pacing_name};
use studyfrog_core::plan::{
    ItemFilter, effect_name, item_kind_name, order_name, state_filter_name,
};
use studyfrog_core::run::{Run, RunState, run_mode_name, run_state_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

pub use crate::run_load::{load_events, load_run};

pub(crate) fn corrupt(what: &str, value: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {value:?}"))
}

pub(crate) fn uid_of(text: &str) -> Result<Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| corrupt("ulid", text))
}

/// Store a new run with its queue and the full plan, filter included.
pub fn insert_run(conn: &Connection, run: &Run) -> Result<(), StoreError> {
    let p = &run.plan;
    let range = p.filter.difficulty;
    if conn.prepare("SELECT typed_sample FROM run LIMIT 0").is_ok() {
        conn.execute(
            "INSERT INTO run (uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again,
                              filter_state, filter_diff_min, filter_diff_max, shuffle, pacing, countdown_secs, filter_fam_below, typed_sample)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                format_ulid(run.uid),
                run_mode_name(run.mode),
                run_state_name(run.state),
                run.started_at.map(|t| t.0),
                order_name(p.order),
                effect_name(p.effect),
                p.seed as i64,
                p.limit,
                p.requeue_again,
                state_filter_name(p.filter.state),
                range.map(|r| r.min),
                range.map(|r| r.max),
                p.shuffle,
                pacing_name(p.pacing),
                countdown_secs(p.pacing),
                p.filter.familiarity_below,
                p.typed_sample.map(|s| s as i64),
            ],
        )
        .map_err(sql_err)?;
    } else {
        conn.execute(
            "INSERT INTO run (uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again,
                              filter_state, filter_diff_min, filter_diff_max, shuffle, pacing, countdown_secs, filter_fam_below)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                format_ulid(run.uid),
                run_mode_name(run.mode),
                run_state_name(run.state),
                run.started_at.map(|t| t.0),
                order_name(p.order),
                effect_name(p.effect),
                p.seed as i64,
                p.limit,
                p.requeue_again,
                state_filter_name(p.filter.state),
                range.map(|r| r.min),
                range.map(|r| r.max),
                p.shuffle,
                pacing_name(p.pacing),
                countdown_secs(p.pacing),
                p.filter.familiarity_below,
            ],
        )
        .map_err(sql_err)?;
    }
    insert_filter_lists(conn, run.uid, &p.filter)?;
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

fn insert_filter_lists(conn: &Connection, run: Uid, f: &ItemFilter) -> Result<(), StoreError> {
    let id = format_ulid(run);
    for (pos, kind) in f.kinds.iter().enumerate() {
        let sql = "INSERT INTO run_filter_kind (run_uid, pos, kind) VALUES (?1, ?2, ?3)";
        conn.execute(sql, params![id, pos as i64, item_kind_name(*kind)])
            .map_err(sql_err)?;
    }
    for (pos, stack) in f.stacks.iter().enumerate() {
        let sql = "INSERT INTO run_filter_stack (run_uid, pos, stack_uid) VALUES (?1, ?2, ?3)";
        conn.execute(sql, params![id, pos as i64, format_ulid(*stack)])
            .map_err(sql_err)?;
    }
    for (pos, tag) in f.tags_any.iter().enumerate() {
        let sql = "INSERT INTO run_filter_tag (run_uid, pos, tag) VALUES (?1, ?2, ?3)";
        conn.execute(sql, params![id, pos as i64, tag])
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
