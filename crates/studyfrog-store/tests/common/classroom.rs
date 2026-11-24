//! A sample classroom session shared by the classroom store tests.

use studyfrog_core::classroom::{
    Assignment, AssignmentKind, ChatTurn, ClassroomSession, GradeReport, GroupRoster, Owner,
    Participant, SourceKind, SourceRef, Submission,
};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

fn name(s: &str) -> String {
    s.to_owned()
}

pub fn teacher(n: &str) -> Participant {
    Participant::Teacher { name: name(n) }
}

pub fn student(n: &str) -> Participant {
    Participant::SimulatedStudent { name: name(n) }
}

pub fn user(n: &str) -> Participant {
    Participant::User { name: name(n) }
}

/// A session with every kind of row: teacher, four students (one of them the user), chat turns,
/// one solo and one group assignment, one graded and one ungraded submission.
pub fn sample(id: u128, title: &str, created: i64) -> ClassroomSession {
    ClassroomSession {
        id: Uid(id),
        title: name(title),
        created_at: UnixMs(created),
        participants: vec![
            teacher("Ms Rowan"),
            student("Ada"),
            student("Ben"),
            user("Cleo"),
            student("Dev"),
        ],
        turns: vec![
            ChatTurn {
                seq: 0,
                speaker: 0,
                at: UnixMs(1_000),
                text: name("Welcome to the lecture."),
            },
            ChatTurn {
                seq: 1,
                speaker: 3,
                at: UnixMs(2_000),
                text: name("What is a cell?"),
            },
            ChatTurn {
                seq: 2,
                speaker: 1,
                at: UnixMs(3_000),
                text: name("The smallest unit of life."),
            },
        ],
        assignments: vec![
            Assignment {
                id: Uid(0xA1),
                title: name("Warm-up"),
                prompt: name("Define a cell."),
                rubric: vec![name("Names the cell's parts")],
                kind: AssignmentKind::Solo { assignee: 3 },
            },
            Assignment {
                id: Uid(0xA2),
                title: name("Project"),
                prompt: name("Draw a cell."),
                rubric: vec![name("Labels every organelle"), name("Shows the membrane")],
                kind: AssignmentKind::Group {
                    groups: vec![
                        GroupRoster {
                            name: name("Group 1"),
                            members: vec![1, 2],
                        },
                        GroupRoster {
                            name: name("Group 2"),
                            members: vec![4, 3],
                        },
                    ],
                },
            },
        ],
        sources: vec![
            SourceRef {
                kind: SourceKind::Stack,
                uid: Uid(77),
            },
            SourceRef {
                kind: SourceKind::Note,
                uid: Uid(78),
            },
        ],
        submissions: vec![
            Submission {
                assignment: Uid(0xA1),
                owner: Owner::Participant(3),
                text: name("A cell is the smallest unit."),
                submitted_at: UnixMs(5_000),
                grade: Some(GradeReport {
                    points: 8,
                    max: 10,
                    feedback: name("Good structure."),
                    graded_at: UnixMs(6_000),
                }),
            },
            Submission {
                assignment: Uid(0xA2),
                owner: Owner::Group(0),
                text: name("Membrane and nucleus."),
                submitted_at: UnixMs(7_000),
                grade: None,
            },
        ],
    }
}
