#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Grading a written exam answer (STORY-0187): the model's judgements are used, and the keyword check is the
//! fallback when there is no model or its reply cannot be read.

use studyfrog_agent::exam_grade::{grade_answer, parse_criteria, share_met};
use studyfrog_agent::rubric_eval::KEYWORD_CAVEAT;

fn rubric() -> Vec<String> {
    vec![
        "names the true score variance".to_owned(),
        "names the observed score variance".to_owned(),
    ]
}

const QUESTION: &str = "What does the reliability coefficient compare?";
const SAMPLE: &str = "True score variance with observed score variance.";

#[test]
fn the_model_judgements_are_used_and_the_share_follows_them() {
    let reply = r#"Stage two done. {"criteria_results": [
        {"criterion_text": "names the true score variance", "met": true, "feedback": "Quoted."},
        {"criterion_text": "names the observed score variance", "met": false, "feedback": "Missing."}
    ]}"#;
    let mut ask = |_: &str, _: &str| Ok(reply.to_owned());
    let g = grade_answer(QUESTION, SAMPLE, &rubric(), "true variance", Some(&mut ask));
    assert!(g.from_model && g.caveat.is_none());
    assert!((share_met(&g.criteria) - 0.5).abs() < 1e-6);
}

#[test]
fn the_model_is_sent_the_question_and_the_answer_only_in_its_prompt() {
    let mut seen = (String::new(), String::new());
    let mut ask = |s: &str, u: &str| {
        seen = (s.to_owned(), u.to_owned());
        Ok("{\"criteria_results\": []}".to_owned())
    };
    let _ = grade_answer(QUESTION, SAMPLE, &rubric(), "my words", Some(&mut ask));
    assert!(seen.0.contains(QUESTION) && seen.0.contains(SAMPLE));
    assert!(seen.0.contains("- names the observed score variance"));
    assert!(seen.1.contains("my words"));
}

#[test]
fn with_no_model_the_keyword_check_grades_and_says_so() {
    let g = grade_answer(
        QUESTION,
        SAMPLE,
        &rubric(),
        "the true score variance and the observed score variance",
        None,
    );
    assert!(!g.from_model);
    assert_eq!(g.caveat.as_deref(), Some(KEYWORD_CAVEAT));
    assert!(g.criteria.iter().all(|c| c.met));
}

#[test]
fn a_failed_model_call_falls_back_to_the_keyword_check_with_a_reason() {
    let mut ask = |_: &str, _: &str| Err("the model is not running".to_owned());
    let g = grade_answer(QUESTION, SAMPLE, &rubric(), "", Some(&mut ask));
    assert!(!g.from_model);
    let caveat = g.caveat.expect("a caveat");
    assert!(caveat.contains("not running") && caveat.contains("keyword check"));
    assert_eq!(share_met(&g.criteria), 0.0);
}

#[test]
fn an_unreadable_reply_falls_back_to_the_keyword_check() {
    let mut ask = |_: &str, _: &str| Ok("I think it is fine.".to_owned());
    let g = grade_answer(
        QUESTION,
        SAMPLE,
        &rubric(),
        "true score variance",
        Some(&mut ask),
    );
    assert!(!g.from_model);
    assert!(g.caveat.is_some_and(|c| c.contains("no JSON object")));
}

#[test]
fn a_rubric_point_the_model_skips_is_not_met() {
    let json = r#"{"criteria_results": [{"criterion_text": "NAMES THE TRUE SCORE VARIANCE", "met": true, "feedback": ""}]}"#;
    let got = parse_criteria(json, &rubric()).unwrap();
    assert!(got[0].met, "matched ignoring case");
    assert!(!got[1].met);
    assert_eq!(got[1].feedback, "Not assessed by the model.");
}

#[test]
fn an_empty_rubric_earns_no_share() {
    assert_eq!(share_met(&[]), 0.0);
}
