#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0236 and TASK-0237: exam answers are scored against fixtures and the exam is graded from what is stored.

mod exam_common;

use exam_common::*;
use studyfrog_core::exam::{ExamGrade, exam_plan};
use studyfrog_core::plan::{ItemKind, plan_sequential};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::compare::score_item_answer;
use studyfrog_service::exam::{exam_result_of, grade_exam_run};
use studyfrog_service::run_history::run_history;
use studyfrog_slm::lexical::Lexical;
use studyfrog_store::answer_scores::load_answer_scores;
use studyfrog_store::exam_results::load_exam_result;

#[test]
fn cards_compare_against_back_and_questions_against_reference_accepted_and_rubric() {
    let dir = fixture_vault("score");
    let s = open(&dir);
    let card = score_item_answer(&s, &Lexical, CARD, "Paris").unwrap();
    assert!(
        card.score.value > 0.99,
        "the back is the reference: {card:?}"
    );
    let wrong = score_item_answer(&s, &Lexical, CARD, "Berlin").unwrap();
    assert!(wrong.score.value < 0.3, "{wrong:?}");
    let synonym = score_item_answer(&s, &Lexical, QUESTION, "powerhouse of the cell").unwrap();
    // Full similarity through the synonym, but no rubric point: 60 % similarity weight, 40 % points.
    assert!(
        (synonym.score.value - 0.6).abs() < 0.01,
        "an accepted synonym counts: {synonym:?}"
    );
    let other = score_item_answer(&s, &Lexical, QUESTION, "the nucleus of a cell").unwrap();
    assert!(other.score.value < synonym.score.value, "{other:?}");
    let points = score_item_answer(&s, &Lexical, QUESTION, "mitochondria make ATP").unwrap();
    assert_eq!(points.score.rubric_hits, [true, true], "the rubric is used");
}

#[test]
fn an_exam_leaves_notes_out_and_grades_all_items_from_the_stored_scores() {
    let dir = fixture_vault("grade");
    let mut s = open(&dir);
    let run = start_exam(&s);
    assert_eq!(run.mode, RunMode::Exam);
    assert_eq!(run.queue.len(), 2, "the note is left out: {:?}", run.queue);
    let answers = [(CARD, "Paris"), (QUESTION, "Berlin is in Germany")];
    play(&mut s, &run, &answers);
    let rows = load_answer_scores(&s.conn, run.uid).unwrap();
    assert_eq!(rows.len(), 2, "every answer was scored and stored");
    let now = UnixMs(1_800_000_100_000);
    let result = grade_exam_run(&s.conn, run.uid, now).unwrap();
    let mean = (rows[0].value + rows[1].value) / 2.0;
    assert!(
        (result.percent - mean * 100.0).abs() <= 0.05,
        "{result:?} vs {mean}"
    );
    assert_eq!((result.items, result.answered), (2, 2));
    assert_eq!(result.passed, result.percent >= 60.0);
    assert_eq!(result.engine, "words");
    assert_eq!(
        load_exam_result(&s.conn, run.uid).unwrap(),
        Some(result.clone())
    );
    let history = run_history(&s.conn).unwrap();
    assert_eq!(history[0].exam, Some(result));
}

#[test]
fn right_answers_earn_an_a_plus_and_unanswered_items_count_zero() {
    let dir = fixture_vault("best");
    let mut s = open(&dir);
    let run = start_exam(&s);
    play(
        &mut s,
        &run,
        &[(CARD, "Paris"), (QUESTION, "The mitochondria make ATP")],
    );
    let best = grade_exam_run(&s.conn, run.uid, UnixMs(1)).unwrap();
    assert_eq!(
        (best.grade, best.passed),
        (ExamGrade::APlus, true),
        "{best:?}"
    );

    let half = start_exam_with(&s, 3);
    play(&mut s, &half, &[(CARD, "Paris")]);
    finish_by_time(&mut s, &half);
    let r = exam_result_of(&s.conn, half.uid, UnixMs(2)).unwrap();
    assert_eq!((r.items, r.answered), (2, 1));
    assert!(r.percent <= 50.0 && r.percent > 45.0, "{r:?}");
    assert_eq!((r.grade, r.passed), (ExamGrade::F, false));
}

#[test]
fn stopped_runs_and_other_modes_are_not_graded() {
    let dir = fixture_vault("refuse");
    let mut s = open(&dir);
    let run = start_exam(&s);
    stop(&mut s, &run);
    assert!(grade_exam_run(&s.conn, run.uid, UnixMs(1)).is_err());
    assert_eq!(load_exam_result(&s.conn, run.uid).unwrap(), None);
    assert!(exam_result_of(&s.conn, Uid(424_242), UnixMs(1)).is_err());
    let plan = exam_plan(plan_sequential()).unwrap();
    assert_eq!(plan.filter.kinds, [ItemKind::Card, ItemKind::Question]);
    assert_eq!(run_state(&s, run.uid), RunState::Aborted);
}

#[test]
fn the_help_has_an_exam_page_linked_from_rehearse_keyboard_and_faq() {
    use studyfrog_service::help::PAGES;
    let page = |id: &str| PAGES.iter().find(|p| p.id == id).unwrap().text;
    let exam = page("exam-mode");
    for needle in [
        "# Exam mode",
        "Submit and lock answer",
        "| 95 to 100 | A+ | 1+ |",
        "| under 60 | F | 5.0 |",
        "60 % or more",
    ] {
        assert!(exam.contains(needle), "{needle}");
    }
    for from in ["rehearse", "keyboard", "faq"] {
        assert!(
            page(from).contains("[[exam-mode]]"),
            "{from} links to the exam page"
        );
    }
}
