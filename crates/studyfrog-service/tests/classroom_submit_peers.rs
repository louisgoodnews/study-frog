#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0292: simulated peer answers for group tasks, with a fake model; no network.

mod classroom_common;
mod classroom_work_common;

use classroom_common::*;
use classroom_work_common::*;

use std::cell::RefCell;

use studyfrog_core::classroom::Owner;
use studyfrog_core::time::UnixMs;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_submit::{record_peer_answers, turn_in};
use studyfrog_service::classroom_task::Assignees;
use studyfrog_service::session::ServiceError;

#[test]
fn simulated_groups_record_their_answers_once_beside_the_users_group() {
    let (_d, s) = vault("submit-peers");
    let id = session_of(&s, four_students());
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "group", Assignees::Groups { size: 2 }, 4);
    turn_in(&lecture, AT, task, 3, "The user's group answer.").unwrap();

    let log = RefCell::new(Vec::new());
    let owners = record_peer_answers(
        &lecture,
        &mut reply_with(
            &log,
            Ok("  Ada and Ben say it protects the cell.  ".to_owned()),
        ),
        UnixMs(6_000),
        task,
    )
    .unwrap();
    assert_eq!(owners, [Owner::Group(0)], "only the group without the user");
    assert!(
        log.borrow()[0].contains("Ada and Ben"),
        "the peer group is named"
    );

    let subs = stored(&s, id).submissions;
    assert_eq!(subs.len(), 2);
    let peer = subs.iter().find(|x| x.owner == Owner::Group(0)).unwrap();
    assert_eq!(
        peer.text, "Ada and Ben say it protects the cell.",
        "trimmed"
    );
    assert_eq!(peer.submitted_at, UnixMs(6_000));
    let mine = subs.iter().find(|x| x.owner == Owner::Group(1)).unwrap();
    assert_eq!(
        mine.text, "The user's group answer.",
        "the user's answer is untouched"
    );
    assert_eq!(mine.submitted_at, AT);

    let again = record_peer_answers(
        &lecture,
        &mut reply_with(&log, Ok("Again.".to_owned())),
        AT,
        task,
    )
    .unwrap();
    assert!(again.is_empty(), "a peer group records its answer once");
    assert_eq!(log.borrow().len(), 1, "no model call for a recorded group");
}

#[test]
fn a_failed_peer_answer_stores_nothing() {
    let (_d, s) = vault("submit-peers-fail");
    let id = session_of(&s, four_students());
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "group", Assignees::Groups { size: 2 }, 4);

    let log = RefCell::new(Vec::new());
    let err = record_peer_answers(
        &lecture,
        &mut reply_with(&log, Err("timed out".to_owned())),
        AT,
        task,
    )
    .unwrap_err();
    assert!(matches!(err, ServiceError::Io(msg) if msg == "timed out"));
    assert!(stored(&s, id).submissions.is_empty(), "nothing is stored");

    let blank = record_peer_answers(
        &lecture,
        &mut reply_with(&log, Ok("  ".to_owned())),
        AT,
        task,
    )
    .unwrap_err();
    assert!(refused_with(blank, "blank"));
    assert!(stored(&s, id).submissions.is_empty());
}

#[test]
fn a_solo_task_has_no_simulated_peers() {
    let (_d, s) = vault("submit-solo-peers");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "solo", Assignees::Solo(1), 3);

    let log = RefCell::new(Vec::new());
    let owners = record_peer_answers(
        &lecture,
        &mut reply_with(&log, Ok("Peer.".to_owned())),
        AT,
        task,
    )
    .unwrap();
    assert!(owners.is_empty());
    assert!(log.borrow().is_empty(), "no model call for a solo task");
}

#[test]
fn peer_answers_are_refused_before_any_model_call_without_consent() {
    let (_d, s) = vault("submit-consent");
    let id = session_of(&s, four_students());
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let task = set_task(&lecture, "group", Assignees::Groups { size: 2 }, 4);

    let cloud = remote(false);
    let refused = Lecture {
        vault: &s,
        provider: &cloud,
        id,
    };
    let log = RefCell::new(Vec::new());
    let err = record_peer_answers(
        &refused,
        &mut reply_with(&log, Ok("Peer.".to_owned())),
        AT,
        task,
    )
    .unwrap_err();
    assert!(refused_with(err, "allow"));
    assert!(log.borrow().is_empty(), "no vault text is sent");
    assert!(stored(&s, id).submissions.is_empty());
}
