#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0292: grading refusals (before turn-in, regrading, wrong owner, unknown task) and group grading.

mod classroom_common;
mod classroom_work_common;

use classroom_common::*;
use classroom_work_common::*;

use std::cell::RefCell;

use studyfrog_agent::lecture_grade::GRADE_MAX;
use studyfrog_core::classroom::Owner;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_grade::grade_submission;
use studyfrog_service::classroom_submit::turn_in;
use studyfrog_service::classroom_task::{Assignees, assign_task};
use studyfrog_service::classroom_turns::start_session;

#[test]
fn grading_before_turn_in_is_refused_with_no_model_call() {
    let (_d, s) = vault("grade-before-turn-in");
    let (id, task) = solo_task_with_answer(&s, false);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let err = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(1),
    )
    .unwrap_err();

    assert!(refused_with(err, "NotSubmitted"));
    assert!(
        log.borrow().is_empty(),
        "no model call for an answer not turned in"
    );
    assert!(stored(&s, id).submissions.is_empty());
}

#[test]
fn an_answer_is_graded_once() {
    let (_d, s) = vault("grade-twice");
    let (id, task) = solo_task_with_answer(&s, true);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(1),
    )
    .unwrap();

    let err = grade_submission(
        &lecture,
        &mut reply_with(&log, Err("never asked".to_owned())),
        UnixMs(9_000),
        task,
        Owner::Participant(1),
    )
    .unwrap_err();
    assert!(refused_with(err, "AlreadyGraded"));
    assert_eq!(
        log.borrow().len(),
        1,
        "the second grading makes no model call"
    );
    let kept = stored(&s, id).submissions[0].grade.clone().unwrap();
    assert_eq!(kept.points, GRADE_MAX, "the first grade stands");
    assert_eq!(kept.graded_at, GRADED_AT);
}

#[test]
fn an_owner_of_the_wrong_kind_is_refused() {
    let (_d, s) = vault("grade-wrong-owner");
    let (id, task) = solo_task_with_answer(&s, true);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let err = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Group(0),
    )
    .unwrap_err();
    assert!(
        refused_with(err, "WrongOwner"),
        "a solo task is graded per student"
    );
    assert!(log.borrow().is_empty());
    assert!(stored(&s, id).submissions[0].grade.is_none());
}

#[test]
fn a_task_that_does_not_exist_is_refused() {
    let (_d, s) = vault("grade-unknown");
    let (id, _task) = solo_task_with_answer(&s, true);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let err = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        Uid(999),
        Owner::Participant(1),
    )
    .unwrap_err();
    assert!(refused_with(err, "UnknownAssignment"));
    assert!(log.borrow().is_empty());
}

#[test]
fn a_group_answer_is_graded_for_its_group_only() {
    let (_d, s) = vault("grade-group");
    let id = start_session(&s, ids(7), "Four", &sources(), four_students()).unwrap();
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let task = assign_task(
        &lecture,
        &mut reply_with(&log, Ok(task_reply("group"))),
        ids(4),
        Assignees::Groups { size: 2 },
    )
    .unwrap();
    turn_in(&lecture, AT, task, 3, ANSWER).unwrap();

    let owner = Owner::Group(1);
    let wrong_member = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Participant(3),
    )
    .unwrap_err();
    assert!(
        refused_with(wrong_member, "WrongOwner"),
        "a group task is graded per group"
    );

    let unknown_group = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        Owner::Group(9),
    )
    .unwrap_err();
    assert!(refused_with(unknown_group, "UnknownGroup"));

    let grade = grade_submission(
        &lecture,
        &mut reply_with(&log, Ok(MODEL_BOTH_MET.to_owned())),
        GRADED_AT,
        task,
        owner,
    )
    .unwrap();
    assert_eq!(grade.label, "A");
    let subs = stored(&s, id).submissions;
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].owner, owner);
    assert_eq!(subs[0].grade.as_ref().unwrap().points, GRADE_MAX);
}
