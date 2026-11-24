#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0291: simulated teammates add contributions to the group thread before the group turns in,
//! with a fake model. Covers what the teammates see, what is stored, and the refusals: a solo task,
//! an unknown group, a group that already turned in, and a refused provider.

mod classroom_common;

use classroom_common::*;

use std::cell::RefCell;

use studyfrog_agent::lecture_prompt::PromptMessage;
use studyfrog_core::classroom::{ClassroomError, GradeReport, Owner};
use studyfrog_core::classroom_ops::append_turn;
use studyfrog_core::classroom_work::{record_grade, submit};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_group::group_contributions;
use studyfrog_service::classroom_task::{Assignees, assign_task};
use studyfrog_service::create::IdSource;
use studyfrog_service::session::{ServiceError, VaultSession};
use studyfrog_store::classroom_store::update_session;

fn say(s: &VaultSession, id: Uid, speaker: usize, text: &str) {
    let mut session = stored(s, id);
    append_turn(&mut session, speaker, AT, text).unwrap();
    update_session(&s.conn, &session).unwrap();
}

#[test]
fn teammates_speak_in_roster_order_and_each_turn_is_stored_before_the_next() {
    let (_d, s) = vault("group-turns");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let said = group_contributions(&lecture, &mut numbered(&log), AT, task, 0).unwrap();
    assert_eq!(said, ["Idea 1.", "Idea 2."]);

    let prompts = log.borrow();
    assert!(prompts[0].contains("You are Ada"));
    assert!(prompts[1].contains("You are Ben"));
    assert!(
        prompts[0].contains("Say what a cell wall does."),
        "the task is in the prompt"
    );
    assert!(
        prompts[1].contains("Ada: Idea 1."),
        "Ben reads Ada's contribution, which is stored first"
    );
    let turns = stored(&s, id).turns;
    let speakers: Vec<usize> = turns.iter().map(|t| t.speaker).collect();
    assert_eq!(speakers, [1, 2]);
    let texts: Vec<&str> = turns.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(texts, ["Idea 1.", "Idea 2."]);
}

#[test]
fn the_group_thread_holds_the_members_turns_and_not_the_others() {
    let (_d, s) = vault("group-thread");
    let id = start(&s);
    let task = with_group_task(&s, id);
    say(&s, id, 0, "Lesson note from the teacher.");
    say(&s, id, 3, "Cleo's idea about the wall.");
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    group_contributions(&lecture, &mut numbered(&log), AT, task, 0).unwrap();
    let prompts = log.borrow();
    assert!(prompts[0].contains("Cleo: Cleo's idea about the wall."));
    assert!(
        !prompts[0].contains("Lesson note"),
        "the teacher's turns are not in the group thread"
    );
}

#[test]
fn a_group_that_turned_in_gets_no_more_contributions_and_its_answer_stays_locked() {
    let (_d, s) = vault("group-turned-in");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let mut session = stored(&s, id);
    let owner = submit(&mut session, task, 3, AT, "Our answer.").unwrap();
    assert_eq!(owner, Owner::Group(0));
    assert_eq!(
        submit(&mut session, task, 1, AT, "A second answer."),
        Err(ClassroomError::AlreadySubmitted)
    );
    assert_eq!(
        submit(&mut session, task, 0, AT, "The teacher's answer."),
        Err(ClassroomError::NotAStudent(0))
    );
    let grade = GradeReport {
        points: 1,
        max: 1,
        feedback: "ok".to_owned(),
        graded_at: UnixMs(9),
        criteria: Vec::new(),
    };
    assert_eq!(
        record_grade(&mut session, task, Owner::Participant(1), grade),
        Err(ClassroomError::WrongOwner)
    );
    update_session(&s.conn, &session).unwrap();

    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let err = group_contributions(&lecture, &mut numbered(&log), AT, task, 0).unwrap_err();
    assert!(matches!(err, ServiceError::Invalid(_)), "{err:?}");
    assert!(log.borrow().is_empty(), "no contribution after turn-in");
    assert!(stored(&s, id).turns.is_empty());
}

#[test]
fn a_solo_task_an_unknown_group_or_an_unknown_task_has_no_teammates() {
    let (_d, s) = vault("group-invalid");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let reply = task_json("solo");
    let mut model = |_: &[PromptMessage]| -> Result<String, String> { Ok(reply.clone()) };
    let solo = assign_task(
        &lecture,
        &mut model,
        IdSource {
            now_ms: 1_000,
            random: [5; 10],
        },
        Assignees::Solo(1),
    )
    .unwrap();
    let log = RefCell::new(Vec::new());
    for (assignment, group) in [(solo, 0), (task, 5), (Uid(999), 0)] {
        let err =
            group_contributions(&lecture, &mut numbered(&log), AT, assignment, group).unwrap_err();
        assert!(matches!(err, ServiceError::Invalid(_)), "{err:?}");
    }
    assert!(log.borrow().is_empty());
    assert!(stored(&s, id).turns.is_empty());
}

#[test]
fn a_remote_provider_without_consent_sends_no_teammate_text() {
    let (_d, s) = vault("group-consent");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let p = remote(false);
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    assert!(group_contributions(&lecture, &mut numbered(&log), AT, task, 0).is_err());
    assert!(log.borrow().is_empty());
}

#[test]
fn a_failing_teammate_stores_no_contribution() {
    let (_d, s) = vault("group-failure");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let mut down = |_: &[PromptMessage]| -> Result<String, String> { Err("down".to_owned()) };
    let err = group_contributions(&lecture, &mut down, AT, task, 0).unwrap_err();
    assert!(matches!(err, ServiceError::Io(_)), "{err:?}");
    assert!(stored(&s, id).turns.is_empty());
}
