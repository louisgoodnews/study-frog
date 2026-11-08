#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The exam rules of the engine: written answers only, locked on submission, strictly one way.

mod common;

use common::{at, go, names, session};
use proptest::prelude::*;
use studyfrog_core::event::{Difficulty, EventKind};
use studyfrog_core::exam::exam_plan;
use studyfrog_core::plan::plan_sequential;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::{Action, StepError};
use studyfrog_rehearsal::session::current_item;
use studyfrog_rehearsal::step::step;

fn exam(queue: &[u128]) -> studyfrog_rehearsal::session::Session {
    session(RunMode::Exam, exam_plan(plan_sequential()).unwrap(), queue)
}

#[test]
fn flip_skip_rate_and_mark_are_refused_and_change_nothing() {
    let mut s = exam(&[1, 2]);
    go(&mut s, 0, Action::Start);
    let before = s.clone();
    for a in [
        Action::Flip,
        Action::Skip,
        Action::Rate(Rating::Good),
        Action::Mark(Difficulty::Easy),
    ] {
        assert_eq!(step(&mut s, at(10, a)), Err(StepError::NotInExam));
    }
    assert_eq!(s, before);
}

#[test]
fn a_blank_answer_cannot_move_the_exam_forward() {
    let mut s = exam(&[1, 2]);
    go(&mut s, 0, Action::Start);
    for blank in ["", "   ", "\n\t"] {
        let r = step(&mut s, at(5, Action::SubmitAnswer(blank.into())));
        assert_eq!(r, Err(StepError::EmptyAnswer));
    }
    assert_eq!(current_item(&s), Some(Uid(1)));
}

#[test]
fn submitting_locks_the_answer_and_shows_the_next_item_until_the_exam_is_finished() {
    let mut s = exam(&[1, 2]);
    go(&mut s, 0, Action::Start);
    let out = go(&mut s, 1500, Action::SubmitAnswer("Paris".into()));
    assert_eq!(names(&out), ["answer_submitted", "item_shown"]);
    assert_eq!(out.latency.unwrap().ms, 1500);
    assert_eq!(current_item(&s), Some(Uid(2)));
    let out = go(&mut s, 2000, Action::SubmitAnswer("56".into()));
    assert_eq!(names(&out), ["answer_submitted", "run_finished"]);
    assert_eq!(s.run.state, RunState::Finished);
}

#[test]
fn other_modes_keep_their_rules() {
    let mut s = session(RunMode::Answer, plan_sequential(), &[1]);
    go(&mut s, 0, Action::Start);
    let out = go(&mut s, 5, Action::SubmitAnswer(String::new()));
    assert_eq!(
        names(&out),
        ["answer_submitted"],
        "answer mode still reveals"
    );
    let mut flip = session(RunMode::Flip, plan_sequential(), &[1]);
    go(&mut flip, 0, Action::Start);
    let refused = step(&mut flip, at(5, Action::SubmitAnswer("x".into())));
    assert_eq!(refused, Err(StepError::WrongMode));
}

fn action() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Start),
        Just(Action::Flip),
        Just(Action::Skip),
        Just(Action::Pause),
        Just(Action::Resume),
        Just(Action::Tick),
        Just(Action::Rate(Rating::Good)),
        Just(Action::Mark(Difficulty::Easy)),
        prop_oneof![Just(String::new()), Just("  ".to_owned()), "[a-z ]{0,6}"]
            .prop_map(Action::SubmitAnswer),
    ]
}

proptest! {
    #[test]
    fn an_exam_is_strictly_one_way_over_any_input_sequence(
        queue in prop::collection::vec(1_u128..50, 0..6).prop_map(|mut q| { q.dedup(); q }),
        steps in prop::collection::vec((0_u32..2000, action()), 0..60),
    ) {
        let mut s = exam(&queue);
        let (mut t, mut cursor, mut shown, mut answered) = (0_u32, 0, Vec::new(), Vec::new());
        for (dt, a) in steps {
            t += dt;
            let Ok(out) = step(&mut s, at(t, a)) else { continue };
            for e in &out.events {
                match &e.kind {
                    EventKind::ItemShown => shown.push(e.item.unwrap()),
                    EventKind::AnswerSubmitted(text) => {
                        prop_assert!(!text.trim().is_empty(), "a blank answer was accepted");
                        answered.push(e.item.unwrap());
                    }
                    EventKind::ItemFlipped | EventKind::ItemSkipped | EventKind::ItemRated(_)
                    | EventKind::ItemMarked(_) => prop_assert!(false, "{:?} in an exam", e.kind),
                    _ => {}
                }
            }
            prop_assert!(s.cursor >= cursor, "the exam went back");
            cursor = s.cursor;
            prop_assert_eq!(s.queue.len(), queue.len(), "the queue grew");
        }
        let want: Vec<Uid> = queue.iter().map(|n| Uid(*n)).collect();
        prop_assert_eq!(&shown[..], &want[..shown.len()], "shown out of order or twice");
        prop_assert_eq!(&answered[..], &want[..answered.len()], "answered out of order or twice");
        prop_assert!(shown.len() <= answered.len() + 1, "an item was passed without an answer");
        if s.run.state == RunState::Finished {
            prop_assert_eq!(answered.len(), queue.len());
        }
    }
}
