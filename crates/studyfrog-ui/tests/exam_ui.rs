#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0238 and TASK-0239: the exam run screen keeps its rules and the result screen shows the grade.

mod common;
mod exam_common;

use common::views::{assert_all_named, harness};
use eframe::egui::Key;
use egui_kittest::kittest::Queryable;
use exam_common::*;
use studyfrog_core::exam::ExamGrade;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunState;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::current_item;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exam_ui::ExamMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

#[test]
fn the_exam_screen_has_an_answer_field_and_no_way_to_skip_peek_or_rate() {
    let mut h = harness(exam_model());
    h.run_steps(2);
    assert!(h.query_by_label("Capital of France?").is_some());
    assert!(h.query_by_label("Exam").is_some(), "the mode pill");
    for absent in [
        "Show answer (Space)",
        "Show answer without answering (Space)",
        "Skip (S)",
        "Good (3)",
        "Easy (E)",
        "Paris",
    ] {
        assert!(
            h.query_by_label(absent).is_none(),
            "{absent} is on the exam screen"
        );
    }
    assert!(h.query_by_label("Your answer").is_some());
    h.get_by_label("Submit and lock answer").click();
    h.run_steps(2);
    assert_eq!(
        current_item(&h.state().0.state.rehearse.run.as_ref().unwrap().session),
        Some(Uid(1)),
        "a blank answer cannot be submitted"
    );
    for key in [Key::Space, Key::S, Key::Num3, Key::E, Key::Escape] {
        h.key_press(key);
        h.run_steps(1);
    }
    let run = h.state().0.state.rehearse.run.as_ref().unwrap();
    assert_eq!(
        (run.session.run.state, current_item(&run.session)),
        (RunState::Running, Some(Uid(1))),
        "keys do not skip, reveal, rate or stop"
    );
    assert!(
        run.events
            .iter()
            .all(|e| e.item.is_none()
                || matches!(e.kind, studyfrog_core::event::EventKind::ItemShown))
    );
    assert_all_named(&h, "exam run");
}

#[test]
fn a_submitted_answer_is_locked_scored_and_the_next_item_follows() {
    let mut m = exam_model();
    let effects = submit(&mut m, "Paris");
    assert!(effects.iter().any(
        |e| matches!(e, Effect::ScoreAnswer { item: Uid(1), answer, .. } if answer == "Paris")
    ));
    let run = m.state.rehearse.run.as_ref().unwrap();
    assert_eq!(current_item(&run.session), Some(Uid(2)));
    assert!(
        run.answer.is_empty(),
        "the field is cleared for the next item"
    );
    for a in [Action::Flip, Action::Skip, Action::Rate(Rating::Easy)] {
        apply(&mut m, Msg::RunAction(a));
    }
    assert_eq!(
        current_item(&m.state.rehearse.run.as_ref().unwrap().session),
        Some(Uid(2))
    );
    apply(&mut m, Msg::AnswerScored(Uid(1), Ok(scored(0.9))));
    assert!(
        m.state
            .rehearse
            .run
            .as_ref()
            .unwrap()
            .scores
            .contains_key(&Uid(1)),
        "a late score is kept in an exam"
    );
}

#[test]
fn grading_starts_once_when_the_last_answer_has_its_score() {
    let mut m = exam_model();
    submit(&mut m, "Paris");
    let last = submit(&mut m, "56");
    assert_eq!(
        m.state.rehearse.run.as_ref().unwrap().session.run.state,
        RunState::Finished
    );
    assert_eq!(
        grade_effects(&last),
        0,
        "not before every answer is compared"
    );
    assert_eq!(
        grade_effects(&apply(&mut m, Msg::AnswerScored(Uid(2), Ok(scored(1.0))))),
        0
    );
    assert_eq!(
        grade_effects(&apply(
            &mut m,
            Msg::AnswerScored(Uid(1), Err("broken".into()))
        )),
        1,
        "a failed comparison counts 0"
    );
    assert_eq!(
        grade_effects(&apply(&mut m, Msg::AnswerScored(Uid(1), Ok(scored(1.0))))),
        0,
        "only once"
    );
    apply(
        &mut m,
        Msg::Exam(ExamMsg::Graded(Uid(100), Ok(result(50.0, ExamGrade::F)))),
    );
    let run = m.state.rehearse.run.as_ref().unwrap();
    assert_eq!(run.exam.result.as_ref().unwrap().grade, ExamGrade::F);
    assert_eq!(
        run.diary_prompt.as_ref().unwrap().note,
        "Exam: 50.0 % (F / 5.0), not passed"
    );
}

#[test]
fn the_result_screen_shows_percent_grade_pass_and_the_review() {
    let mut h = harness(graded_model());
    h.run_steps(3);
    for text in [
        "Exam finished",
        "82.5 %",
        "B+ / 1.7",
        "Passed",
        "Gut (good)",
        "Your answer: Paris",
        "Reference: Paris",
        "Your answer: 54",
        "Reference: 56",
        "100 % (Correct)",
        "65 % (Not quite)",
    ] {
        assert!(h.query_by_label(text).is_some(), "{text} is missing");
    }
    assert_all_named(&h, "exam result");
    h.get_by_label("Back to setup").click();
    h.run_steps(2);
    assert!(h.state().0.state.rehearse.run.is_none());
}

#[test]
fn a_stopped_exam_is_not_graded() {
    let mut m = exam_model();
    submit(&mut m, "Paris");
    let effects = apply(&mut m, Msg::RunAction(Action::Abort));
    assert_eq!(grade_effects(&effects), 0);
    assert_eq!(
        grade_effects(&apply(&mut m, Msg::AnswerScored(Uid(1), Ok(scored(1.0))))),
        0
    );
    let mut h = harness(m);
    h.run_steps(3);
    assert!(h.query_by_label("Exam stopped").is_some());
    assert!(h.query_by_label("Not answered (counts 0)").is_some());
}
