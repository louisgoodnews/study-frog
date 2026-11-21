#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the rubric evaluator: model JSON, missing criteria, scoring, and the offline fallback.

use studyfrog_agent::rubric_eval::{fallback_evaluate, overall_pct, parse_model_report};

fn rubric() -> Vec<String> {
    vec![
        "cites criterion validity".to_owned(),
        "addresses fabbing risk".to_owned(),
    ]
}

#[test]
fn model_json_is_parsed_and_scored_by_this_crate() {
    let json = r#"{"choice_valid": true, "criteria_results": [
        {"criterion_text": "Cites criterion validity", "met": true, "feedback": "Good."},
        {"criterion_text": "addresses fabbing risk", "met": false, "feedback": "Mention faking."}],
        "overall_score": 100}"#;
    let r = parse_model_report(json, &rubric()).unwrap();
    assert!(r.from_model);
    assert!(r.criteria[0].met);
    assert!(!r.criteria[1].met);
    assert!((r.overall_pct - 70.0).abs() < 1e-3, "{}", r.overall_pct);
}

#[test]
fn a_criterion_the_model_skipped_counts_as_not_met() {
    let json = r#"{"choice_valid": false, "criteria_results": []}"#;
    let r = parse_model_report(json, &rubric()).unwrap();
    assert!(r.criteria.iter().all(|c| !c.met));
    assert_eq!(r.criteria[0].feedback, "Not assessed by the model.");
    assert!(r.overall_pct.abs() < 1e-3);
}

#[test]
fn malformed_model_output_is_an_error() {
    assert!(parse_model_report("{not json", &rubric()).is_err());
}

#[test]
fn scores_weight_the_choice_and_the_rubric() {
    assert!((overall_pct(true, &[]) - 100.0).abs() < 1e-3);
    assert!((overall_pct(false, &[]) - 60.0).abs() < 1e-3);
}

#[test]
fn offline_fallback_meets_criteria_by_their_key_words() {
    let answer = "Concurrent validation shows criterion validity, because the test scores match \
                  job performance; respondents might be fabbing answers, a risk to address.";
    let r = fallback_evaluate(answer, true, &rubric());
    assert!(!r.from_model);
    assert!(r.criteria[0].met, "{:?}", r.criteria[0]);
    assert!(r.criteria[1].met, "{:?}", r.criteria[1]);
    assert!((r.overall_pct - 100.0).abs() < 1e-3);
}

#[test]
fn offline_fallback_names_missing_ideas() {
    let r = fallback_evaluate("It is a good test.", false, &rubric());
    assert!(!r.criteria[0].met);
    assert!(
        r.criteria[0]
            .feedback
            .starts_with("Not yet covered. Consider:")
    );
    assert!(r.overall_pct < 1.0);
}
