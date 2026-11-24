//! Where classroom work stands (EPIC-068, TASK-0294): owners of a task, the status of each owner and of
//! the task as the user sees it, and the grade lookups the screen reads.

use super::classroom::{grade, group, people, solo};
use crate::classroom::{ClassroomSession, Owner, Participant};
use crate::classroom_ops::new_session;
use crate::classroom_status::{
    WorkStatus, grade_for, owner_of, owners_of, status_of, submission_for, task_status,
};
use crate::classroom_work::{assign, record_grade, submit};
use crate::time::UnixMs;
use crate::uid::Uid;

/// Two more simulated students than [`people`], so two groups of two can be made.
fn wide_session() -> ClassroomSession {
    let mut all = people();
    all.push(Participant::SimulatedStudent { name: "Cy".into() });
    all.push(Participant::SimulatedStudent { name: "Di".into() });
    new_session(Uid(1), "Lecture 1", UnixMs(0), all).expect("valid session")
}

#[test]
fn a_solo_task_moves_from_assigned_to_submitted_to_graded() {
    let mut s = wide_session();
    assign(&mut s, solo(1, 1)).unwrap();
    assert_eq!(task_status(&s, Uid(1), Some(1)), WorkStatus::Assigned);
    submit(&mut s, Uid(1), 1, UnixMs(5), "My answer").unwrap();
    assert_eq!(task_status(&s, Uid(1), Some(1)), WorkStatus::Submitted);
    record_grade(&mut s, Uid(1), Owner::Participant(1), grade(7, 10)).unwrap();
    assert_eq!(task_status(&s, Uid(1), Some(1)), WorkStatus::Graded);
    assert_eq!(
        grade_for(&s, Uid(1), Owner::Participant(1)).unwrap().points,
        7
    );
    assert_eq!(
        submission_for(&s, Uid(1), Owner::Participant(1))
            .unwrap()
            .text,
        "My answer"
    );
}

#[test]
fn a_group_task_shows_the_users_own_group_and_the_least_advanced_group_otherwise() {
    let mut s = wide_session();
    // The user (3) is in group 0 with Ann; group 1 is Ben and Cy.
    assign(&mut s, group(2, vec![vec![3, 1], vec![2, 4]])).unwrap();
    submit(&mut s, Uid(2), 1, UnixMs(5), "Our answer").unwrap();
    record_grade(&mut s, Uid(2), Owner::Group(0), grade(9, 10)).unwrap();
    assert_eq!(owner_of(&s, Uid(2), 1), Some(Owner::Group(0)));
    assert_eq!(owner_of(&s, Uid(2), 2), Some(Owner::Group(1)));
    assert_eq!(task_status(&s, Uid(2), Some(3)), WorkStatus::Graded);
    assert_eq!(task_status(&s, Uid(2), Some(2)), WorkStatus::Assigned);
    assert_eq!(task_status(&s, Uid(2), None), WorkStatus::Assigned);
    assert_eq!(
        owners_of(&s, Uid(2)),
        vec![Owner::Group(0), Owner::Group(1)]
    );
    assert_eq!(status_of(&s, Uid(2), Owner::Group(1)), WorkStatus::Assigned);
}

#[test]
fn a_student_without_the_task_has_no_owner_and_an_unknown_task_has_none() {
    let mut s = wide_session();
    assign(&mut s, solo(3, 2)).unwrap();
    assert_eq!(owner_of(&s, Uid(3), 1), None);
    assert_eq!(owner_of(&s, Uid(99), 2), None);
    assert!(owners_of(&s, Uid(99)).is_empty());
    assert_eq!(task_status(&s, Uid(3), Some(1)), WorkStatus::Assigned);
}
