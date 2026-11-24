#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Teacher grading of a turned-in answer and the simulated group's peer answer (STORY-0210, TASK-0292).
//! Pure: no model is called; replies are fixed strings.

use studyfrog_agent::lecture_grade::{
    GRADE_MAX, grade_answer, grade_messages, label_of, parse_grade, points_of,
};
use studyfrog_agent::lecture_peer::peer_messages;
use studyfrog_agent::lecture_prompt::PromptRole;
use studyfrog_agent::rubric_eval::CriterionResult;

fn rubric() -> Vec<String> {
    vec![
        "Names the function".to_owned(),
        "Gives an example".to_owned(),
    ]
}

const BOTH_MET: &str = r#"{"criteria": [{"criterion": "names the function", "met": true, "feedback": "Clear."}, {"criterion": "Gives an example", "met": true, "feedback": "Good example."}], "summary": "Well argued. Keep the example."}"#;

#[test]
fn a_model_reply_gives_points_label_criteria_and_summary() {
    let grade = parse_grade(BOTH_MET, &rubric()).unwrap();
    assert_eq!(grade.points, GRADE_MAX);
    assert_eq!(grade.label, "A");
    assert!(grade.from_model);
    assert_eq!(grade.caveat, None);
    assert_eq!(grade.summary, "Well argued. Keep the example.");
    assert_eq!(grade.criteria.len(), 2);
    assert_eq!(grade.criteria[0].criterion, "Names the function");
    assert!(grade.criteria[0].met && grade.criteria[0].checked);
    assert_eq!(grade.criteria[1].feedback, "Good example.");
}

#[test]
fn a_rubric_point_the_reply_does_not_mention_is_not_met() {
    let reply = r#"{"criteria": [{"criterion": "Names the function", "met": true, "feedback": "Yes."}], "summary": "Partly there."}"#;
    let grade = parse_grade(reply, &rubric()).unwrap();
    assert_eq!(grade.points, 50);
    assert_eq!(grade.label, "F");
    assert!(!grade.criteria[1].met);
    assert_eq!(grade.criteria[1].feedback, "Not assessed by the model.");
}

#[test]
fn json_inside_prose_still_parses() {
    let reply = format!("Here is my grade: {BOTH_MET} Thank you.");
    assert_eq!(parse_grade(&reply, &rubric()).unwrap().points, GRADE_MAX);
}

#[test]
fn a_reply_that_cannot_be_read_is_refused() {
    let rubric = rubric();
    assert!(parse_grade("no grade here", &rubric).is_err(), "no JSON");
    assert!(parse_grade("{not json", &rubric).is_err(), "malformed JSON");
    let blank_summary =
        r#"{"criteria": [{"criterion": "Names the function", "met": true}], "summary": "  "}"#;
    assert!(
        parse_grade(blank_summary, &rubric).is_err(),
        "blank summary"
    );
    let other_points =
        r#"{"criteria": [{"criterion": "Something else", "met": true}], "summary": "Fine."}"#;
    assert!(
        parse_grade(other_points, &rubric).is_err(),
        "no rubric point named"
    );
}

#[test]
fn a_model_error_gives_the_keyword_grade_with_its_reason() {
    let grade = grade_answer(
        Err("connection refused"),
        "It names the function only.",
        &rubric(),
    );
    assert!(!grade.from_model);
    assert_eq!(
        grade.points, 50,
        "the keyword check finds the first point only"
    );
    assert_eq!(grade.label, "F");
    let caveat = grade.caveat.unwrap();
    assert!(caveat.contains("connection refused"));
    assert!(caveat.contains("keyword check"));
}

#[test]
fn an_unreadable_reply_gives_the_keyword_grade_too() {
    let grade = grade_answer(
        Ok("I cannot grade this."),
        "It names the function and gives an example.",
        &rubric(),
    );
    assert!(!grade.from_model);
    assert_eq!(grade.points, GRADE_MAX);
    assert!(grade.caveat.unwrap().contains("could not grade"));
}

#[test]
fn a_readable_reply_is_used_over_the_keyword_check() {
    let grade = grade_answer(Ok(BOTH_MET), "It names the function only.", &rubric());
    assert!(grade.from_model);
    assert_eq!(grade.points, GRADE_MAX, "the model judged both points met");
}

#[test]
fn points_count_only_checked_criteria_and_round_down() {
    let crit = |met: bool, checked: bool| CriterionResult {
        criterion: "c".to_owned(),
        met,
        checked,
        feedback: String::new(),
    };
    assert_eq!(
        points_of(&[crit(true, true), crit(false, true), crit(true, true)]),
        66
    );
    assert_eq!(
        points_of(&[crit(true, true), crit(false, false)]),
        100,
        "unchecked points do not count"
    );
    assert_eq!(
        points_of(&[crit(true, false)]),
        0,
        "nothing checked scores 0"
    );
    assert_eq!(points_of(&[]), 0);
}

#[test]
fn letters_follow_the_score_bands() {
    assert_eq!(label_of(100), "A");
    assert_eq!(label_of(90), "A");
    assert_eq!(label_of(89), "B");
    assert_eq!(label_of(80), "B");
    assert_eq!(label_of(79), "C");
    assert_eq!(label_of(70), "C");
    assert_eq!(label_of(69), "D");
    assert_eq!(label_of(60), "D");
    assert_eq!(label_of(59), "F");
    assert_eq!(label_of(0), "F");
}

#[test]
fn the_grading_prompt_carries_the_task_rubric_and_answer() {
    let messages = grade_messages(
        "Dr Lin",
        "Cell walls",
        "Say what a cell wall does.",
        &rubric(),
        "It protects the cell.",
    );
    assert_eq!(messages[0].role, PromptRole::System);
    let system = &messages[0].content;
    assert!(system.contains("Dr Lin"));
    assert!(system.contains("Cell walls"));
    assert!(system.contains("- Names the function"));
    assert!(system.contains("- Gives an example"));
    assert!(system.contains("\"criteria\""), "the reply shape is named");
    assert_eq!(messages[1].role, PromptRole::User);
    assert!(messages[1].content.contains("It protects the cell."));
}

#[test]
fn the_peer_prompt_names_the_group_and_the_rubric() {
    let messages = peer_messages(
        "Cell walls",
        "Say what a cell wall does.",
        &rubric(),
        &["Ada", "Ben"],
    );
    let system = &messages[0].content;
    assert!(system.contains("Ada and Ben"));
    assert!(system.contains("\"Cell walls\""));
    assert!(system.contains("- Gives an example"));
    assert_eq!(messages[1].role, PromptRole::User);
}
