use crate::classroom::{
    Assignment, AssignmentKind, ClassroomError, ClassroomSession, GradeReport, GroupRoster,
    Participant,
};
use crate::classroom_ops::{append_turn, new_session};
use crate::time::UnixMs;
use crate::uid::Uid;

pub(super) fn people() -> Vec<Participant> {
    vec![
        Participant::Teacher {
            name: "Prof".into(),
        },
        Participant::SimulatedStudent { name: "Ann".into() },
        Participant::SimulatedStudent { name: "Ben".into() },
        Participant::User { name: "Me".into() },
    ]
}

pub(super) fn session() -> ClassroomSession {
    new_session(Uid(1), "Lecture 1", UnixMs(0), people()).expect("valid session")
}

pub(super) fn solo(id: u128, assignee: usize) -> Assignment {
    Assignment {
        id: Uid(id),
        title: "Quiz".into(),
        prompt: "Explain X".into(),
        rubric: vec!["Accurate".into()],
        kind: AssignmentKind::Solo { assignee },
    }
}

pub(super) fn group(id: u128, groups: Vec<Vec<usize>>) -> Assignment {
    Assignment {
        id: Uid(id),
        title: "Project".into(),
        prompt: "Build Y".into(),
        rubric: vec!["Accurate".into()],
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

pub(super) fn grade(points: u32, max: u32) -> GradeReport {
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
