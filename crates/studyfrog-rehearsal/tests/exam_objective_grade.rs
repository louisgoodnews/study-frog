#![allow(clippy::unwrap_used)]
//! Tests for deterministic objective grading across single-select, multiple-select, and fill-in-the-gaps (STORY-0207).

use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::exam_answer::{
    ExamAnswer, grade_fill_in_gaps, grade_multiple_select, grade_objective, grade_single_select,
    is_objective_format,
};
use studyfrog_rehearsal::exam_format::ExamItemFormat;

fn opts() -> Vec<String> {
    vec![
        "Alpha".to_owned(),
        "Beta".to_owned(),
        "Gamma".to_owned(),
        "Delta".to_owned(),
    ]
}

#[test]
fn single_select_awards_all_or_zero() {
    let o = opts();
    assert_eq!(grade_single_select(Some(1), &[1], &o, 2).earned, 2);
    assert_eq!(grade_single_select(Some(0), &[1], &o, 2).earned, 0);
    assert_eq!(grade_single_select(None, &[1], &o, 2).earned, 0);
}

#[test]
fn multiple_select_applies_net_partial_credit_with_zero_floor() {
    let o = opts();
    // Correct set = {0, 2} (2 correct choices), worth 4 points.
    assert_eq!(
        grade_multiple_select(&[true, false, true, false], &[0, 2], &o, 4).earned,
        4
    );
    // 1 hit, 0 wrong -> 1/2 = 2 points.
    assert_eq!(
        grade_multiple_select(&[true, false, false, false], &[0, 2], &o, 4).earned,
        2
    );
    // 1 hit, 1 wrong -> max(0, 1 - 1)/2 = 0 points.
    assert_eq!(
        grade_multiple_select(&[true, true, false, false], &[0, 2], &o, 4).earned,
        0
    );
    // All boxes checked -> max(0, 2 - 2)/2 = 0 points.
    assert_eq!(
        grade_multiple_select(&[true, true, true, true], &[0, 2], &o, 4).earned,
        0
    );
    // Nothing checked -> 0 points.
    assert_eq!(
        grade_multiple_select(&[false, false, false, false], &[0, 2], &o, 4).earned,
        0
    );
}

#[test]
fn fill_in_gaps_normalizes_case_whitespace_and_trailing_punctuation() {
    let expected = vec!["consistency".to_owned(), "true score".to_owned()];
    let exact = grade_fill_in_gaps(
        &["  Consistency. ".to_owned(), "TRUE   score;".to_owned()],
        &expected,
        4,
    );
    assert_eq!(exact.earned, 4);
    assert!(exact.checks.iter().all(|(_, ok)| *ok));

    let half = grade_fill_in_gaps(
        &["consistency".to_owned(), "wrong".to_owned()],
        &expected,
        4,
    );
    assert_eq!(half.earned, 2);
}

#[test]
fn open_calculation_and_case_are_not_graded_as_objective() {
    for fmt in [
        ExamItemFormat::OpenQuestion,
        ExamItemFormat::Calculation,
        ExamItemFormat::CaseScenario,
    ] {
        assert!(!is_objective_format(fmt));
        let key = AnswerKey {
            format: fmt,
            prompt: "Q".to_owned(),
            sample_answer: "A".to_owned(),
            rubric: vec!["A".to_owned()],
            correct_indices: Vec::new(),
            gap_answers: Vec::new(),
        };
        assert_eq!(
            grade_objective(fmt, &ExamAnswer::Text("A".to_owned()), &key, &[], 2),
            None
        );
    }
}
