#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for case-study validation and the choice check.

use studyfrog_rehearsal::case_study::{CaseStudyItem, choice_is_correct, validate_case_study};

fn case() -> CaseStudyItem {
    CaseStudyItem {
        vignette: "A hiring manager needs a short screen for analysts; the test must predict job performance."
            .to_owned(),
        decision_prompt: "Which procedure should be used to check criterion validity?".to_owned(),
        options: vec!["Concurrent validation".to_owned(), "Test-retest".to_owned(), "Cronbach".to_owned()],
        expected_choices: vec!["Concurrent validation".to_owned()],
        rubric_criteria: vec!["cites criterion validity".to_owned(), "addresses fabbing risk".to_owned()],
    }
}

#[test]
fn a_well_formed_case_validates() {
    assert!(validate_case_study(&case()).is_ok());
}

#[test]
fn short_vignettes_and_empty_prompts_are_refused() {
    let mut short = case();
    short.vignette = "Too short.".to_owned();
    assert!(validate_case_study(&short).is_err());
    let mut blank = case();
    blank.decision_prompt = " ".to_owned();
    assert!(validate_case_study(&blank).is_err());
}

#[test]
fn options_and_expected_choices_must_agree() {
    let mut one = case();
    one.options.truncate(1);
    assert!(validate_case_study(&one).is_err());
    let mut dup = case();
    dup.options[1] = "concurrent validation".to_owned();
    assert!(validate_case_study(&dup).is_err());
    let mut unknown = case();
    unknown.expected_choices = vec!["Factor analysis".to_owned()];
    assert!(validate_case_study(&unknown).is_err());
}

#[test]
fn rubric_must_be_present_bounded_and_distinct() {
    let mut none = case();
    none.rubric_criteria.clear();
    assert!(validate_case_study(&none).is_err());
    let mut many = case();
    many.rubric_criteria = (0..13).map(|i| format!("point {i}")).collect();
    assert!(validate_case_study(&many).is_err());
    let mut twice = case();
    twice.rubric_criteria[1] = "Cites Criterion Validity".to_owned();
    assert!(validate_case_study(&twice).is_err());
}

#[test]
fn choice_check_ignores_order_and_case() {
    let mut c = case();
    c.expected_choices = vec!["Concurrent validation".to_owned(), "Test-retest".to_owned()];
    assert!(choice_is_correct(
        &c,
        &["test-retest".to_owned(), "CONCURRENT VALIDATION".to_owned()]
    ));
    assert!(!choice_is_correct(&c, &["Test-retest".to_owned()]));
}
