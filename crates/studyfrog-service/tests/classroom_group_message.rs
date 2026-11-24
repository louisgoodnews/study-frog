#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The user's group thread (EPIC-068, TASK-0294): the user writes to the group and the simulated teammates
//! answer after it, with a fake model. Refused for the teacher, for a blank message, and with no model call.

mod classroom_common;

use classroom_common::*;

use std::cell::RefCell;

use studyfrog_core::classroom_work::submit;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_group::group_message;
use studyfrog_service::session::ServiceError;
use studyfrog_store::classroom_store::update_session;

#[test]
fn the_user_writes_to_the_group_and_the_teammates_answer_after_it() {
    let (_d, s) = vault("group-message");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let said = group_message(
        &lecture,
        &mut numbered(&log),
        AT,
        task,
        0,
        "  Let's split the work.  ",
    )
    .unwrap();
    assert_eq!(said, ["Idea 1.", "Idea 2."]);
    assert!(
        log.borrow()[0].contains("Let's split the work."),
        "the teammates read the user's message first"
    );
    let turns = stored(&s, id).turns;
    let speakers: Vec<usize> = turns.iter().map(|t| t.speaker).collect();
    assert_eq!(speakers, [3, 1, 2]);
    assert_eq!(
        turns[0].text, "Let's split the work.",
        "trimmed before it is stored"
    );
}

#[test]
fn a_blank_message_or_one_for_a_turned_in_group_is_refused_before_any_model_call() {
    let (_d, s) = vault("group-message-refused");
    let id = start(&s);
    let task = with_group_task(&s, id);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let err = group_message(&lecture, &mut numbered(&log), AT, task, 0, "   ").unwrap_err();
    assert!(matches!(err, ServiceError::Invalid(_)), "{err:?}");
    assert!(log.borrow().is_empty(), "no teammate is asked");
    assert!(stored(&s, id).turns.is_empty(), "nothing is stored");

    let mut session = stored(&s, id);
    submit(&mut session, task, 3, AT, "Our answer.").unwrap();
    update_session(&s.conn, &session).unwrap();
    let err = group_message(&lecture, &mut numbered(&log), AT, task, 0, "Too late?").unwrap_err();
    assert!(matches!(err, ServiceError::Invalid(_)), "{err:?}");
    assert!(
        log.borrow().is_empty(),
        "no teammate is asked after turn-in"
    );
    assert!(
        stored(&s, id).turns.is_empty(),
        "the late message is not stored"
    );
}
