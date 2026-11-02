//! Recording a run: persist events and, when the plan asks for it, update the schedule.

use rusqlite::Connection;
use studyfrog_core::event::EventKind;
use studyfrog_core::plan::{RunPlan, ScheduleEffect};
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::VaultSettings;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_rehearsal::session::Session;
use studyfrog_store::error::{StoreError, sql_err};
use studyfrog_store::runs::{abort_unfinished_runs, append_events, insert_run, set_run_state};
use studyfrog_store::schedule::{
    insert_review, load_mem_state, load_sm2_state, save_mem_state, save_sm2_state,
};
use studyfrog_store::settings::item_home;

use studyfrog_md::ulid::compose_ulid;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;

use crate::create::IdSource;
use crate::day::DayWindow;
use crate::familiarity::{retime_fsrs, retime_sm2};
use crate::queue::{NewRun, new_run};
use crate::schedule::{apply_rating, apply_rating_sm2};
use crate::session::{ServiceError, VaultSession, current_settings};
use crate::settings::{params_for, scheduler_for};

/// Store a new run and its queue.
pub fn begin_run(conn: &Connection, run: &Run) -> Result<(), StoreError> {
    insert_run(conn, run)
}

/// What `record_step` needs to know about the run: its identity, what it does to the schedule and
/// its lifecycle state after the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepContext {
    /// The run.
    pub run: Uid,
    /// Whether ratings change the schedule.
    pub effect: ScheduleEffect,
    /// Lifecycle state after the step.
    pub state: RunState,
}

/// The context of a session after a step.
pub fn step_context(session: &Session) -> StepContext {
    StepContext {
        run: session.run.uid,
        effect: session.run.plan.effect,
        state: session.run.state,
    }
}

/// Persist one engine step in a single transaction: events, lifecycle state, and (for
/// `UpdateSchedule` plans) a review plus new schedule state per rating, through each item's own
/// scheduler. `now` is the wall-clock time of the step.
pub fn record_step(
    conn: &mut Connection,
    ctx: &StepContext,
    out: &StepOutput,
    now: UnixMs,
    vault: &VaultSettings,
) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    append_events(&tx, &out.events)?;
    let started = out
        .events
        .iter()
        .any(|e| e.kind == EventKind::RunStarted)
        .then_some(now);
    set_run_state(&tx, ctx.run, ctx.state, started)?;
    if ctx.effect == ScheduleEffect::UpdateSchedule {
        for e in &out.events {
            if let (EventKind::ItemRated(rating), Some(item)) = (&e.kind, e.item) {
                rate_item(&tx, item, *rating, now, vault)?;
            }
        }
    }
    tx.commit().map_err(sql_err)
}

pub(crate) fn rate_item(
    conn: &Connection,
    item: Uid,
    rating: Rating,
    now: UnixMs,
    vault: &VaultSettings,
) -> Result<(), StoreError> {
    let home = item_home(conn, item)?;
    match scheduler_for(vault, home.as_ref()) {
        Scheduler::Fsrs => {
            let params = params_for(vault, home.and_then(|h| h.request_retention));
            let prev = load_mem_state(conn, item)?;
            let (review, mut mem) = apply_rating(prev.as_ref(), item, rating, now, &params);
            retime_fsrs(
                conn,
                &mut mem,
                (item, rating),
                (&params, vault, home.as_ref()),
            )?;
            insert_review(conn, &review)?;
            save_mem_state(conn, item, &mem)
        }
        Scheduler::Sm2 => {
            let prev = load_sm2_state(conn, item)?;
            let (review, mut state) = apply_rating_sm2(
                prev.as_ref(),
                item,
                rating,
                now,
                vault.maximum_interval_days,
            );
            retime_sm2(conn, &mut state, (item, rating), (vault, home.as_ref()))?;
            insert_review(conn, &review)?;
            save_sm2_state(conn, item, &state)
        }
    }
}

/// Call once at start-up: runs left open by a crash become aborted. Returns how many.
pub fn recover_runs(conn: &mut Connection) -> Result<usize, StoreError> {
    abort_unfinished_runs(conn)
}

/// True once a run can no longer change.
pub fn is_over(run: &Run) -> bool {
    matches!(run.state, RunState::Finished | RunState::Aborted)
}

/// Create and store a run: the queue comes from the index, the run id from `ids`, and the shuffle
/// seed from its random bytes (so a shuffled run can be reproduced from what is stored).
pub fn start_run(
    session: &VaultSession,
    plan: RunPlan,
    mode: RunMode,
    now: UnixMs,
    day: DayWindow,
    ids: IdSource,
) -> Result<Run, ServiceError> {
    let seed = u64::from_le_bytes([
        ids.random[0],
        ids.random[1],
        ids.random[2],
        ids.random[3],
        ids.random[4],
        ids.random[5],
        ids.random[6],
        ids.random[7],
    ]);
    let req = NewRun {
        plan: RunPlan { seed, ..plan },
        mode,
        uid: compose_ulid(ids.now_ms, ids.random),
        now,
        day,
        global_budget: budget_unlimited(),
    };
    let vault = current_settings(session);
    let run = new_run(&session.conn, req, &vault).map_err(ServiceError::Store)?;
    begin_run(&session.conn, &run).map_err(ServiceError::Store)?;
    Ok(run)
}
