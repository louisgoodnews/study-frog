#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0292: turn-in stores a solo or group answer trimmed and locked, filed under the owner; refusals

mod classroom_common;
mod classroom_work_common;

use classroom_common::*;
use classroom_work_common::*;

use studyfrog_core::classroom::{AssignmentKind, Owner};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_submit::turn_in;
use studyfrog_service::classroom_task::Assignees;

#[test]
fn a_solo_answer_is_stored_trimmed_and_locked() {
    let (_d, s) = vault("submit-solo");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "solo", Assignees::Solo(1), 3);

    let owner = turn_in(&lecture, AT, task, 1, "  Cell walls protect the cell.  ").unwrap();
    assert_eq!(owner, Owner::Participant(1));

    let stored_session = stored(&s, id);
    assert_eq!(stored_session.submissions.len(), 1);
    let sub = &stored_session.submissions[0];
    assert_eq!(sub.text, "Cell walls protect the cell.");
    assert_eq!(sub.submitted_at, AT);
    assert_eq!(sub.grade, None, "a turned-in answer is not graded yet");

    let again = turn_in(&lecture, UnixMs(9_000), task, 1, "A changed answer.").unwrap_err();
    assert!(
        refused_with(again, "AlreadySubmitted"),
        "the answer is locked"
    );
    let kept = &stored(&s, id).submissions[0];
    assert_eq!(kept.text, "Cell walls protect the cell.");
    assert_eq!(kept.submitted_at, AT);
}

#[test]
fn only_the_assignee_can_turn_in_a_solo_answer() {
    let (_d, s) = vault("submit-wrong-owner");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "solo", Assignees::Solo(1), 3);

    let err = turn_in(&lecture, AT, task, 3, "The user's answer.").unwrap_err();
    assert!(
        refused_with(err, "NotAssigned"),
        "the user is not the assignee"
    );
    assert!(stored(&s, id).submissions.is_empty(), "nothing is stored");
}

#[test]
fn a_blank_answer_and_an_unknown_task_are_refused() {
    let (_d, s) = vault("submit-refused");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "solo", Assignees::Solo(1), 3);

    let blank = turn_in(&lecture, AT, task, 1, "   \n").unwrap_err();
    assert!(refused_with(blank, "empty"));
    let unknown = turn_in(&lecture, AT, Uid(999), 1, "An answer.").unwrap_err();
    assert!(refused_with(unknown, "UnknownAssignment"));
    assert!(stored(&s, id).submissions.is_empty());
}

#[test]
fn a_group_answer_is_filed_under_the_members_group() {
    let (_d, s) = vault("submit-group");
    let id = session_of(&s, four_students());
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "group", Assignees::Groups { size: 2 }, 4);

    let groups = match &stored(&s, id).assignments[0].kind {
        AssignmentKind::Group { groups } => groups.clone(),
        AssignmentKind::Solo { .. } => panic!("a group task is stored as a group"),
    };
    assert_eq!(groups[0].members, [1, 2]);
    assert_eq!(groups[1].members, [3, 4]);

    let owner = turn_in(&lecture, AT, task, 3, "Group answer.").unwrap();
    assert_eq!(owner, Owner::Group(1), "Cleo is in group 1");
    let owner = turn_in(&lecture, AT, task, 1, "Ada's group answer.").unwrap();
    assert_eq!(owner, Owner::Group(0), "Ada is in group 0");

    let subs = stored(&s, id).submissions;
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].owner, Owner::Group(1));
    assert_eq!(subs[0].text, "Group answer.");
    let second_turn = turn_in(&lecture, AT, task, 4, "Dan again.").unwrap_err();
    assert!(
        refused_with(second_turn, "AlreadySubmitted"),
        "the group has turned in"
    );
}

#[test]
fn the_teacher_cannot_turn_in_work() {
    let (_d, s) = vault("submit-teacher");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "solo", Assignees::Solo(1), 3);

    let err = turn_in(&lecture, AT, task, 0, "The teacher's answer.").unwrap_err();
    assert!(refused_with(err, "NotAStudent"));
}
