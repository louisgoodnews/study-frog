#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The run state machine: scripted runs, latency, answers and refusals.

mod common;

use common::{at, go, names, session};
use studyfrog_core::event::{EventKind, Latency, LatencyKind, event_name};
use studyfrog_core::plan::plan_sequential;
use studyfrog_core::review::Rating::{Easy, Good};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::clock::{Clock, ManualClock, advance, input_now};
use studyfrog_rehearsal::input::{Action, Input, StepError};
use studyfrog_rehearsal::session::Session;
use studyfrog_rehearsal::step::step;

#[test]
fn scripted_flip_run_produces_the_expected_log() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[1, 2]);
    let mut log = Vec::new();
    let mut lat = Vec::new();
    for (t, a) in [
        (0, Action::Start),
        (1500, Action::Flip),
        (2000, Action::Rate(Good)),
        (2600, Action::Flip),
        (3000, Action::Rate(Easy)),
    ] {
        let out = go(&mut s, t, a);
        log.extend(out.events);
        lat.extend(out.latency);
    }
    let summary: Vec<_> = log
        .iter()
        .map(|e| (e.seq, e.t_ms, e.item.map(|u| u.0), event_name(&e.kind)))
        .collect();
    assert_eq!(
        summary,
        [
            (0, 0, None, "run_started"),
            (1, 0, Some(1), "item_shown"),
            (2, 1500, Some(1), "item_flipped"),
            (3, 2000, Some(1), "item_rated"),
            (4, 2000, Some(2), "item_shown"),
            (5, 2600, Some(2), "item_flipped"),
            (6, 3000, Some(2), "item_rated"),
            (7, 3000, None, "run_finished"),
        ]
    );
    assert_eq!(
        lat,
        [
            Latency {
                item: Uid(1),
                kind: LatencyKind::Flip,
                ms: 1500
            },
            Latency {
                item: Uid(2),
                kind: LatencyKind::Flip,
                ms: 600
            },
        ]
    );
    assert_eq!(s.run.state, RunState::Finished);
}

#[test]
fn paused_time_is_excluded_from_latency_and_resets_per_item() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[1, 2]);
    go(&mut s, 0, Action::Start);
    go(&mut s, 1000, Action::Pause);
    go(&mut s, 4000, Action::Resume);
    go(&mut s, 4500, Action::Pause);
    go(&mut s, 5000, Action::Resume);
    let flip = go(&mut s, 5500, Action::Flip);
    assert_eq!(flip.latency.unwrap().ms, 5500 - 3000 - 500);
    go(&mut s, 6000, Action::Pause);
    go(&mut s, 9000, Action::Resume);
    go(&mut s, 9100, Action::Rate(Good));
    let second = go(&mut s, 9400, Action::Skip);
    assert_eq!(
        second.latency.unwrap().ms,
        300,
        "item 1's pauses do not leak into item 2"
    );
}

#[test]
fn answer_mode_records_the_text() {
    let mut s = session(RunMode::Answer, plan_sequential(), &[1]);
    go(&mut s, 0, Action::Start);
    let out = go(&mut s, 2200, Action::SubmitAnswer("Paris".into()));
    assert_eq!(
        out.events[0].kind,
        EventKind::AnswerSubmitted("Paris".into())
    );
    assert_eq!(
        out.latency,
        Some(Latency {
            item: Uid(1),
            kind: LatencyKind::Answer,
            ms: 2200
        })
    );
    assert_eq!(
        names(&go(&mut s, 3000, Action::Rate(Good))),
        ["item_rated", "run_finished"]
    );
}

#[test]
fn refused_inputs_leave_the_session_unchanged() {
    let cases: Vec<(&str, Session, Input, StepError)> = {
        let idle = session(RunMode::Flip, plan_sequential(), &[1]);
        let mut running = idle.clone();
        go(&mut running, 100, Action::Start);
        let mut revealed = running.clone();
        go(&mut revealed, 200, Action::Flip);
        let mut paused = running.clone();
        go(&mut paused, 200, Action::Pause);
        let mut ended = running.clone();
        go(&mut ended, 200, Action::Abort);
        vec![
            (
                "flip before start",
                idle.clone(),
                at(0, Action::Flip),
                StepError::NotStarted,
            ),
            (
                "start twice",
                running.clone(),
                at(300, Action::Start),
                StepError::AlreadyStarted,
            ),
            (
                "rate before reveal",
                running.clone(),
                at(300, Action::Rate(Good)),
                StepError::NotRevealed,
            ),
            (
                "flip twice",
                revealed.clone(),
                at(300, Action::Flip),
                StepError::NotAwaitingReveal,
            ),
            (
                "skip after reveal",
                revealed,
                at(300, Action::Skip),
                StepError::NotAwaitingReveal,
            ),
            (
                "resume when running",
                running.clone(),
                at(300, Action::Resume),
                StepError::NotPaused,
            ),
            (
                "answer in flip mode",
                running.clone(),
                at(300, Action::SubmitAnswer("x".into())),
                StepError::WrongMode,
            ),
            (
                "flip while paused",
                paused,
                at(300, Action::Flip),
                StepError::Paused,
            ),
            (
                "time goes back",
                running,
                at(50, Action::Pause),
                StepError::TimeWentBackwards,
            ),
            (
                "anything after abort",
                ended,
                at(300, Action::Pause),
                StepError::Ended,
            ),
        ]
    };
    for (label, mut s, input, expected) in cases {
        let before = s.clone();
        assert_eq!(step(&mut s, input), Err(expected), "{label}");
        assert_eq!(s, before, "{label}: session changed");
    }
}

#[test]
fn empty_queue_finishes_immediately_and_abort_works_from_idle() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[]);
    assert_eq!(
        names(&go(&mut s, 0, Action::Start)),
        ["run_started", "run_finished"]
    );
    let mut idle = session(RunMode::Flip, plan_sequential(), &[1]);
    assert_eq!(names(&go(&mut idle, 0, Action::Abort)), ["run_aborted"]);
    assert_eq!(step(&mut idle, at(1, Action::Start)), Err(StepError::Ended));
}

#[test]
fn manual_clock_drives_the_engine() {
    let clock = ManualClock::default();
    let mut s = session(RunMode::Flip, plan_sequential(), &[1]);
    step(&mut s, input_now(&clock, Action::Start)).unwrap();
    advance(&clock, 750);
    assert_eq!(clock.now_ms(), 750);
    let out = step(&mut s, input_now(&clock, Action::Flip)).unwrap();
    assert_eq!(out.latency.unwrap().ms, 750);
}
