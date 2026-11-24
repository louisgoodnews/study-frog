#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0292: the teacher grades a turned-in answer through a fake model, and the keyword check when the model fails.

mod classroom_common;
mod classroom_work_common;

use classroom_common::*;
use classroom_work_common::*;

use std::cell::RefCell;
use studyfrog_agent::lecture_grade::GRADE_MAX;
use studyfrog_core::classroom::Owner;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_grade::grade_submission;

#[test]
fn the_model_grade_is_recorded_with_points_label_and_summary() {
    let (_d, s) = vault("grade-model");
    let (id, task) = solo_task_with_answer(&s, true);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let grade = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(1),
    )
    .unwrap();

    assert!(grade.from_model);
    assert_eq!(grade.caveat, None);
    assert_eq!(grade.points, GRADE_MAX);
    assert_eq!(grade.label, "A");
    assert_eq!(grade.criteria.len(), 2);
    assert_eq!(grade.criteria[1].feedback, "A good example.");
    assert!(log.borrow()[0].contains("Dr Lin"), "the teacher grades");
    assert!(
        log.borrow()[0].contains("- Gives an example"),
        "against the rubric"
    );

    let stored_grade = stored(&s, id).submissions[0].grade.clone().unwrap();
    assert_eq!(stored_grade.points, GRADE_MAX);
    assert_eq!(stored_grade.max, GRADE_MAX);
    assert_eq!(stored_grade.feedback, "Well argued overall.");
    assert_eq!(stored_grade.graded_at, GRADED_AT);
}

#[test]
fn a_failed_model_call_grades_by_the_keyword_check() {
    let (_d, s) = vault("grade-fallback-error");
    let (id, task) = solo_task_with_answer(&s, true);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let grade = grade_submission(
        &lecture,
        &mut reply_with(&log, Err("connection refused".to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(1),
    )
    .unwrap();

    assert!(!grade.from_model);
    assert_eq!(
        grade.points, 50,
        "the answer names the function, not the example"
    );
    assert_eq!(grade.label, "F");
    let caveat = grade.caveat.unwrap();
    assert!(caveat.contains("connection refused"));
    assert!(caveat.contains("keyword check"));

    let stored_grade = stored(&s, id).submissions[0].grade.clone().unwrap();
    assert_eq!(stored_grade.points, 50);
    assert_eq!(
        stored_grade.feedback,
        "The keyword check found 1 of 2 checked rubric points in the answer."
    );
}

#[test]
fn a_reply_that_does_not_read_grades_by_the_keyword_check() {
    let (_d, s) = vault("grade-fallback-malformed");
    let (id, task) = solo_task_with_answer(&s, true);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let grade = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok("{\"criteria\": [broken".to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(1),
    )
    .unwrap();

    assert!(!grade.from_model);
    assert_eq!(grade.points, 50);
    assert!(grade.caveat.unwrap().contains("could not grade"));
    assert!(
        stored(&s, id).submissions[0].grade.is_some(),
        "the answer is graded, not left open"
    );
}

#[test]
fn a_provider_that_may_not_receive_vault_text_is_not_asked() {
    let (_d, s) = vault("grade-consent");
    let (id, task) = solo_task_with_answer(&s, true);
    let cloud = remote(false);
    let lecture = Lecture {
        vault: &s,
        provider: &cloud,
        id,
    };
    let log = RefCell::new(Vec::new());
    let grade = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(1),
    )
    .unwrap();

    assert!(log.borrow().is_empty(), "no prompt is sent");
    assert!(!grade.from_model);
    assert!(grade.caveat.unwrap().contains("allow"));
    assert!(stored(&s, id).submissions[0].grade.is_some());
}
