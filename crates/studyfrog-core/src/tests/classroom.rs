use crate::classroom::{
    Assignment, AssignmentKind, ClassroomError, ClassroomSession, GradeReport, GroupRoster, Owner,
    Participant,
};
use crate::classroom_ops::{append_turn, new_session};
use crate::classroom_work::{assign, record_grade, submit};
use crate::time::UnixMs;
use crate::uid::Uid;

fn people() -> Vec<Participant> {
    vec![
        Participant::Teacher {
            name: "Prof".into(),
        },
        Participant::SimulatedStudent { name: "Ann".into() },
        Participant::SimulatedStudent { name: "Ben".into() },
        Participant::User { name: "Me".into() },
    ]
}

fn session() -> ClassroomSession {
    new_session(Uid(1), "Lecture 1", UnixMs(0), people()).expect("valid session")
}

fn solo(id: u128, assignee: usize) -> Assignment {
    Assignment {
        id: Uid(id),
        title: "Quiz".into(),
        prompt: "Explain X".into(),
        kind: AssignmentKind::Solo { assignee },
    }
}

fn group(id: u128, groups: Vec<Vec<usize>>) -> Assignment {
    Assignment {
        id: Uid(id),
        title: "Project".into(),
        prompt: "Build Y".into(),
        kind: AssignmentKind::Group {
            groups: groups
                .into_iter()
                .enumerate()
                .map(|(i, members)| GroupRoster {
                    name: format!("G{i}"),
                    members,
                })
                .collect(),
        },
    }
}

fn grade(points: u32, max: u32) -> GradeReport {
    GradeReport {
        points,
        max,
        feedback: "ok".into(),
        graded_at: UnixMs(9),
    }
}

#[test]
fn a_session_needs_a_teacher_and_non_blank_names() {
    let no_teacher = vec![Participant::User { name: "Me".into() }];
    assert_eq!(
        new_session(Uid(1), "T", UnixMs(0), no_teacher),
        Err(ClassroomError::NoTeacher)
    );
    assert_eq!(
        new_session(Uid(1), "  ", UnixMs(0), people()),
        Err(ClassroomError::EmptyText)
    );
    let blank_name = vec![Participant::Teacher { name: "".into() }];
    assert_eq!(
        new_session(Uid(1), "T", UnixMs(0), blank_name),
        Err(ClassroomError::EmptyText)
    );
}

#[test]
fn turns_are_numbered_and_must_not_go_back_in_time() {
    let mut s = session();
    append_turn(&mut s, 0, UnixMs(10), "Welcome").unwrap();
    append_turn(&mut s, 1, UnixMs(10), "Hi").unwrap();
    assert_eq!(s.turns[1].seq, 1);
    assert_eq!(
        append_turn(&mut s, 2, UnixMs(5), "late"),
        Err(ClassroomError::TurnOutOfOrder)
    );
    assert_eq!(s.turns.len(), 2, "a refused turn leaves the chat unchanged");
}

#[test]
fn a_turn_needs_a_real_speaker_and_text() {
    let mut s = session();
    assert_eq!(
        append_turn(&mut s, 9, UnixMs(0), "x"),
        Err(ClassroomError::UnknownParticipant(9))
    );
    assert_eq!(
        append_turn(&mut s, 0, UnixMs(0), " "),
        Err(ClassroomError::EmptyText)
    );
    assert!(s.turns.is_empty());
}

#[test]
fn a_solo_assignment_goes_to_a_student_not_the_teacher() {
    let mut s = session();
    assert_eq!(
        assign(&mut s, solo(1, 0)),
        Err(ClassroomError::NotAStudent(0))
    );
    assign(&mut s, solo(1, 1)).unwrap();
    assert_eq!(
        assign(&mut s, solo(1, 2)),
        Err(ClassroomError::DuplicateAssignment(Uid(1)))
    );
}

#[test]
fn group_rosters_need_two_distinct_students_and_no_shared_member() {
    let mut s = session();
    assert_eq!(
        assign(&mut s, group(1, vec![vec![1]])),
        Err(ClassroomError::GroupTooSmall)
    );
    assert_eq!(
        assign(&mut s, group(1, vec![vec![1, 1]])),
        Err(ClassroomError::DuplicateMember(1))
    );
    assert_eq!(
        assign(&mut s, group(1, vec![vec![1, 2], vec![2, 3]])),
        Err(ClassroomError::DuplicateMember(2))
    );
    assert_eq!(
        assign(&mut s, group(1, vec![vec![0, 1]])),
        Err(ClassroomError::NotAStudent(0))
    );
    assert_eq!(
        assign(&mut s, group(1, vec![])),
        Err(ClassroomError::GroupTooSmall)
    );
    assign(&mut s, group(1, vec![vec![1, 2], vec![3]])).expect_err("a group of one is refused");
    assign(&mut s, group(1, vec![vec![1, 2]])).unwrap();
}

#[test]
fn assignment_text_must_not_be_blank() {
    let mut s = session();
    let mut a = solo(1, 1);
    a.prompt = "   ".into();
    assert_eq!(assign(&mut s, a), Err(ClassroomError::EmptyText));
}

#[test]
fn a_solo_assignee_turns_in_once_and_the_answer_is_locked() {
    let mut s = session();
    assign(&mut s, solo(1, 1)).unwrap();
    let owner = submit(&mut s, Uid(1), 1, UnixMs(5), "My answer").unwrap();
    assert_eq!(owner, Owner::Participant(1));
    assert_eq!(
        submit(&mut s, Uid(1), 1, UnixMs(6), "Edited"),
        Err(ClassroomError::AlreadySubmitted)
    );
    assert_eq!(s.submissions[0].text, "My answer");
}

#[test]
fn only_the_assignee_may_turn_in_solo_work() {
    let mut s = session();
    assign(&mut s, solo(1, 1)).unwrap();
    assert_eq!(
        submit(&mut s, Uid(1), 2, UnixMs(5), "Stolen"),
        Err(ClassroomError::NotAssigned(2))
    );
    assert_eq!(
        submit(&mut s, Uid(99), 1, UnixMs(5), "x"),
        Err(ClassroomError::UnknownAssignment(Uid(99)))
    );
}

#[test]
fn a_group_member_turns_in_for_the_whole_group() {
    let mut s = session();
    assign(&mut s, group(1, vec![vec![1, 2]])).unwrap();
    assert_eq!(
        submit(&mut s, Uid(1), 2, UnixMs(5), "Team answer").unwrap(),
        Owner::Group(0)
    );
    assert_eq!(
        submit(&mut s, Uid(1), 1, UnixMs(6), "Again"),
        Err(ClassroomError::AlreadySubmitted)
    );
}

#[test]
fn grading_needs_a_turn_in_first() {
    let mut s = session();
    assign(&mut s, solo(1, 1)).unwrap();
    assert_eq!(
        record_grade(&mut s, Uid(1), Owner::Participant(1), grade(5, 10)),
        Err(ClassroomError::NotSubmitted)
    );
    submit(&mut s, Uid(1), 1, UnixMs(5), "Answer").unwrap();
    record_grade(&mut s, Uid(1), Owner::Participant(1), grade(7, 10)).unwrap();
    assert_eq!(s.submissions[0].grade.as_ref().map(|g| g.points), Some(7));
}

#[test]
fn a_grade_is_recorded_once_and_must_fit_the_maximum() {
    let mut s = session();
    assign(&mut s, solo(1, 1)).unwrap();
    submit(&mut s, Uid(1), 1, UnixMs(5), "Answer").unwrap();
    assert_eq!(
        record_grade(&mut s, Uid(1), Owner::Participant(1), grade(11, 10)),
        Err(ClassroomError::GradeOutOfRange {
            points: 11,
            max: 10
        })
    );
    assert_eq!(
        record_grade(&mut s, Uid(1), Owner::Participant(1), grade(0, 0)),
        Err(ClassroomError::GradeOutOfRange { points: 0, max: 0 })
    );
    record_grade(&mut s, Uid(1), Owner::Participant(1), grade(8, 10)).unwrap();
    assert_eq!(
        record_grade(&mut s, Uid(1), Owner::Participant(1), grade(9, 10)),
        Err(ClassroomError::AlreadyGraded)
    );
}

#[test]
fn a_grade_must_name_an_owner_of_the_assignments_kind() {
    let mut s = session();
    assign(&mut s, solo(1, 1)).unwrap();
    assign(&mut s, group(2, vec![vec![1, 2]])).unwrap();
    assert_eq!(
        record_grade(&mut s, Uid(1), Owner::Group(0), grade(1, 1)),
        Err(ClassroomError::WrongOwner)
    );
    assert_eq!(
        record_grade(&mut s, Uid(2), Owner::Participant(1), grade(1, 1)),
        Err(ClassroomError::WrongOwner)
    );
    assert_eq!(
        record_grade(&mut s, Uid(2), Owner::Group(3), grade(1, 1)),
        Err(ClassroomError::UnknownGroup(3))
    );
}
