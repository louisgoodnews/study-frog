//! Recording a run: persist events and, when the plan asks for it, update the schedule.

use rusqlite::Connection;
use studyfrog_core::event::EventKind;
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunState};
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

use crate::schedule::{apply_rating, apply_rating_sm2};
use crate::settings::{params_for, scheduler_for};

/// Store a new run and its queue.
pub fn begin_run(conn: &Connection, run: &Run) -> Result<(), StoreError> {
    insert_run(conn, run)
}

/// Persist one engine step in a single transaction: events, lifecycle state, and (for
/// `UpdateSchedule` plans) a review plus new schedule state per rating, through each item's own
/// scheduler. `now` is the wall-clock time of the step; `session` is the session after the step.
pub fn record_step(
    conn: &mut Connection,
    session: &Session,
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
    set_run_state(&tx, session.run.uid, session.run.state, started)?;
    if session.run.plan.effect == ScheduleEffect::UpdateSchedule {
        for e in &out.events {
            if let (EventKind::ItemRated(rating), Some(item)) = (&e.kind, e.item) {
                rate_item(&tx, item, *rating, now, vault)?;
            }
        }
    }
    tx.commit().map_err(sql_err)
}

fn rate_item(
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
            let (review, mem) = apply_rating(prev.as_ref(), item, rating, now, &params);
            insert_review(conn, &review)?;
            save_mem_state(conn, item, &mem)
        }
        Scheduler::Sm2 => {
            let prev = load_sm2_state(conn, item)?;
            let (review, state) = apply_rating_sm2(
                prev.as_ref(),
                item,
                rating,
                now,
                vault.maximum_interval_days,
            );
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
