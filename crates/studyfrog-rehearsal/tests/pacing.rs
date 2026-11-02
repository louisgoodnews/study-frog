#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Countdown and count up: the clock, the end of a countdown and what it leaves unanswered.

mod common;

use common::{go, names, session};
use proptest::prelude::*;
use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::pacing::Pacing;
use studyfrog_core::plan::{RunPlan, plan_sequential};
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::{Action, StepError};
use studyfrog_rehearsal::pacing::{active_ms, remaining_ms, run_active_ms};
use studyfrog_rehearsal::session::Session;
use studyfrog_rehearsal::step::step;

fn plan(pacing: Pacing) -> RunPlan {
    RunPlan {
        pacing,
        ..plan_sequential()
    }
}

fn countdown(queue: &[u128]) -> Session {
    session(RunMode::Flip, plan(Pacing::Countdown(60)), queue)
}

fn rated(events: &[RunEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::ItemRated(_)))
        .count()
}

#[test]
fn a_countdown_ends_by_itself_when_the_time_is_up_and_the_open_item_is_not_answered() {
    let mut s = countdown(&[1, 2, 3]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 5_000, Action::Flip);
    go(&mut s, 6_000, Action::Rate(Good));
    assert_eq!(remaining_ms(&s, 30_000), Some(30_000));
    assert!(
        go(&mut s, 59_999, Action::Tick).events.is_empty(),
        "one millisecond early changes nothing"
    );
    let out = go(&mut s, 60_000, Action::Tick);
    assert_eq!(names(&out), ["time_up", "run_finished"]);
    assert_eq!(
        out.events[0].item,
        Some(Uid(2)),
        "the item on screen is the one left unanswered"
    );
    assert_eq!(rated(&out.events), 0, "no rating, so no schedule change");
    assert_eq!(s.run.state, RunState::Finished);
    assert_eq!(
        step(&mut s, common::at(61_000, Action::Tick)),
        Err(StepError::Ended)
    );
}

#[test]
fn an_item_whose_answer_was_shown_but_not_rated_is_unanswered_too() {
    let mut s = countdown(&[1, 2]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 58_000, Action::Flip);
    let out = go(&mut s, 60_500, Action::Tick);
    assert_eq!(names(&out), ["time_up", "run_finished"]);
    assert_eq!(out.events[0].item, Some(Uid(1)));
}

#[test]
fn an_input_at_or_after_the_limit_is_not_applied_the_clock_owns_that_second() {
    let mut s = countdown(&[1, 2]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 59_000, Action::Flip);
    let out = go(&mut s, 60_000, Action::Rate(Good));
    assert_eq!(names(&out), ["time_up", "run_finished"]);
    assert_eq!(rated(&out.events), 0);
    let mut late = countdown(&[1]);
    go(&mut late, 0, Action::Start);
    let out = go(&mut late, 90_000, Action::Abort);
    assert_eq!(
        names(&out),
        ["time_up", "run_finished"],
        "even leaving the run is the clock's"
    );
}

#[test]
fn pausing_stops_the_clock_and_a_paused_run_never_runs_out() {
    let mut s = countdown(&[1, 2]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 50_000, Action::Pause);
    assert!(
        go(&mut s, 400_000, Action::Tick).events.is_empty(),
        "ticks do nothing while paused"
    );
    assert_eq!(active_ms(&s, 400_000), 50_000);
    assert_eq!(remaining_ms(&s, 400_000), Some(10_000));
    go(&mut s, 450_000, Action::Resume);
    assert_eq!(active_ms(&s, 459_999), 59_999, "the pause is not counted");
    assert!(go(&mut s, 459_999, Action::Tick).events.is_empty());
    assert_eq!(
        names(&go(&mut s, 460_000, Action::Tick)),
        ["time_up", "run_finished"]
    );
}

#[test]
fn the_clock_starts_when_the_run_does_not_when_the_session_is_made() {
    let mut s = countdown(&[1]);
    go(&mut s, 10_000, Action::Start);
    assert_eq!(active_ms(&s, 10_000), 0);
    assert!(go(&mut s, 69_999, Action::Tick).events.is_empty());
    assert_eq!(
        names(&go(&mut s, 70_000, Action::Tick)),
        ["time_up", "run_finished"]
    );
}

#[test]
fn a_queue_used_up_early_finishes_normally_and_again_items_keep_coming_until_the_time_is_up() {
    let mut early = countdown(&[1]);
    go(&mut early, 0, Action::Start);
    go(&mut early, 1_000, Action::Flip);
    let out = go(&mut early, 2_000, Action::Rate(Good));
    assert_eq!(
        names(&out),
        ["item_rated", "run_finished"],
        "no time_up: the list was done"
    );

    let mut again = session(
        RunMode::Flip,
        RunPlan {
            requeue_again: true,
            ..plan(Pacing::Countdown(60))
        },
        &[1],
    );
    go(&mut again, 0, Action::Start);
    go(&mut again, 1_000, Action::Flip);
    let out = go(&mut again, 2_000, Action::Rate(Again));
    assert_eq!(
        names(&out),
        ["item_rated", "item_shown"],
        "the lapse comes back"
    );
    assert_eq!(
        names(&go(&mut again, 61_000, Action::Tick)),
        ["time_up", "run_finished"]
    );
}

#[test]
fn free_and_count_up_runs_never_time_out() {
    for pacing in [Pacing::Free, Pacing::CountUp] {
        let mut s = session(RunMode::Flip, plan(pacing), &[1, 2]);
        go(&mut s, 0, Action::Start);
        assert!(
            go(&mut s, 3_600_000, Action::Tick).events.is_empty(),
            "{pacing:?}"
        );
        assert_eq!(remaining_ms(&s, 3_600_000), None);
        assert_eq!(
            active_ms(&s, 3_600_000),
            3_600_000,
            "a count up can show its elapsed time"
        );
        assert_eq!(s.run.state, RunState::Running);
    }
}

#[test]
fn a_count_up_ends_when_the_last_item_is_completed_or_when_the_user_ends_it() {
    let mut s = session(RunMode::Flip, plan(Pacing::CountUp), &[1]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 9_000, Action::Flip);
    assert_eq!(
        names(&go(&mut s, 10_000, Action::Rate(Good))),
        ["item_rated", "run_finished"]
    );
    let mut early = session(RunMode::Flip, plan(Pacing::CountUp), &[1, 2]);
    go(&mut early, 0, Action::Start);
    assert_eq!(
        names(&go(&mut early, 5_000, Action::Abort)),
        ["run_aborted"]
    );
}

#[test]
fn a_tick_needs_a_started_run() {
    let mut s = countdown(&[1]);
    assert_eq!(
        step(&mut s, common::at(0, Action::Tick)),
        Err(StepError::NotStarted)
    );
}

#[test]
fn the_time_a_run_used_is_read_from_its_events_without_the_time_paused() {
    let mut s = countdown(&[1, 2, 3]);
    let mut log = go(&mut s, 0, Action::Start).events;
    log.extend(go(&mut s, 10_000, Action::Pause).events);
    log.extend(go(&mut s, 40_000, Action::Resume).events);
    log.extend(go(&mut s, 45_000, Action::Flip).events);
    log.extend(go(&mut s, 50_000, Action::Rate(Good)).events);
    assert_eq!(run_active_ms(&log), 20_000);
    assert_eq!(run_active_ms(&[]), 0);
    log.extend(go(&mut s, 55_000, Action::Pause).events);
    assert_eq!(
        run_active_ms(&log),
        25_000,
        "a pause that never ended counts up to the last event"
    );
}

proptest! {
    #[test]
    fn a_countdown_ends_once_never_rates_after_the_end_and_has_used_at_least_its_limit(
        steps in proptest::collection::vec((0_u32..30_000, 0_u8..8), 1..60),
    ) {
        let mut s = countdown(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let mut log = go(&mut s, 0, Action::Start).events;
        let mut t = 0;
        for (dt, kind) in steps {
            t += dt;
            let action = match kind {
                0 => Action::Flip,
                1 => Action::Skip,
                2 => Action::Rate(Good),
                3 | 4 => Action::Tick,
                5 => Action::Pause,
                6 => Action::Resume,
                _ => Action::Tick,
            };
            if let Ok(out) = step(&mut s, common::at(t, action)) {
                log.extend(out.events);
            }
        }
        let time_ups = log.iter().filter(|e| e.kind == EventKind::TimeUp).count();
        prop_assert!(time_ups <= 1);
        if let Some(at) = log.iter().position(|e| e.kind == EventKind::TimeUp) {
            prop_assert_eq!(log.len(), at + 2, "time_up and run_finished are the last two events");
            prop_assert!(run_active_ms(&log) >= 60_000, "a countdown only ends when the time is used");
        }
    }
}
