#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0291: the teacher sets solo and group tasks with a fake model. Covers persistence of the
//! prompt, rubric and assignees, the fallback when the reply does not parse, and the refusals that
//! make no model call.

mod classroom_common;

use classroom_common::*;

use std::cell::RefCell;

use studyfrog_agent::lecture_prompt::PromptMessage;
use studyfrog_core::classroom::AssignmentKind;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_task::{Assignees, assign_task};
use studyfrog_service::create::IdSource;
use studyfrog_service::session::ServiceError;

/// The teacher's JSON reply for a task of `mode`.
fn task_json(mode: &str) -> String {
    format!(
        r#"{{"title": "Cell walls", "prompt": "Say what a cell wall does.", "mode": "{mode}", "rubric": ["Names the function", "Gives an example"]}}"#
    )
}

fn ids(random: u8) -> IdSource {
    IdSource {
        now_ms: 1_000,
        random: [random; 10],
    }
}

/// A fake model that logs each prompt (system then user) and always gives `reply`.
fn answer<'a>(
    log: &'a RefCell<Vec<String>>,
    reply: String,
) -> impl FnMut(&[PromptMessage]) -> Result<String, String> + 'a {
    move |messages| {
        log.borrow_mut()
            .push(format!("{}\n{}", messages[0].content, messages[1].content));
        Ok(reply.clone())
    }
}

#[test]
fn a_solo_task_is_grounded_in_the_material_and_stored_with_its_rubric() {
    let (_d, s) = vault("task-solo");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let task = assign_task(
        &lecture,
        &mut answer(&log, task_json("solo")),
        ids(3),
        Assignees::Solo(1),
    )
    .unwrap();

    let prompts = log.borrow();
    assert!(prompts[0].contains("One student will work on it alone"));
    assert!(
        prompts[0].contains("Mitochondria"),
        "the task follows the current segment"
    );
    let assignments = stored(&s, id).assignments;
    assert_eq!(assignments.len(), 1);
    let a = &assignments[0];
    assert_eq!(a.id, task);
    assert_eq!(a.title, "Cell walls");
    assert_eq!(a.prompt, "Say what a cell wall does.");
    assert_eq!(a.rubric, ["Names the function", "Gives an example"]);
    assert_eq!(a.kind, AssignmentKind::Solo { assignee: 1 });
}

#[test]
fn a_group_task_puts_the_user_and_the_students_into_a_stored_group() {
    let (_d, s) = vault("task-group");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    assign_task(
        &lecture,
        &mut answer(&log, task_json("group")),
        ids(4),
        Assignees::Groups { size: 2 },
    )
    .unwrap();

    let stored_task = stored(&s, id).assignments.remove(0);
    let AssignmentKind::Group { groups } = stored_task.kind else {
        panic!("a group task is stored as a group");
    };
    assert_eq!(
        groups.len(),
        1,
        "three students with size two make one group"
    );
    assert_eq!(groups[0].name, "Group 1");
    assert_eq!(
        groups[0].members,
        [1, 2, 3],
        "Ada, Ben and the user; never the teacher"
    );
    assert!(log.borrow()[0].contains("A group of students will hand in one shared answer"));
}

#[test]
fn a_reply_that_does_not_parse_gives_the_fixed_fallback_task() {
    let (_d, s) = vault("task-fallback");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    assign_task(
        &lecture,
        &mut answer(&log, "Sorry, I cannot write a task.".to_owned()),
        ids(5),
        Assignees::Solo(2),
    )
    .unwrap();
    // A task of the other mode is not usable either, so the fallback is used.
    assign_task(
        &lecture,
        &mut answer(&log, task_json("group")),
        ids(6),
        Assignees::Solo(2),
    )
    .unwrap();

    let assignments = stored(&s, id).assignments;
    assert_eq!(assignments.len(), 2);
    for a in &assignments {
        assert!(
            a.title.starts_with("Explain "),
            "fallback title: {}",
            a.title
        );
        assert!(a.prompt.contains("In your own words"));
        assert!(!a.rubric.is_empty());
        assert_eq!(a.kind, AssignmentKind::Solo { assignee: 2 });
    }
    assert_eq!(
        assignments[0].prompt, assignments[1].prompt,
        "the fallback is deterministic"
    );
}

#[test]
fn two_tasks_are_stored_side_by_side_with_their_own_ids() {
    let (_d, s) = vault("task-two");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let first = assign_task(
        &lecture,
        &mut answer(&log, task_json("solo")),
        ids(7),
        Assignees::Solo(1),
    )
    .unwrap();
    let second = assign_task(
        &lecture,
        &mut answer(&log, task_json("group")),
        ids(8),
        Assignees::Groups { size: 2 },
    )
    .unwrap();
    assert_ne!(first, second);
    let assignments = stored(&s, id).assignments;
    let ids: Vec<_> = assignments.iter().map(|a| a.id).collect();
    assert_eq!(ids, [first, second]);
}

#[test]
fn refused_assignees_and_providers_make_no_model_call_and_store_nothing() {
    let (_d, s) = vault("task-refused");
    let id = start(&s);
    let local_p = local();
    let remote_p = remote(false);
    let log = RefCell::new(Vec::new());
    let cases = [
        (&local_p, Assignees::Solo(0)),
        (&local_p, Assignees::Solo(9)),
        (&local_p, Assignees::Groups { size: 1 }),
        (&remote_p, Assignees::Solo(1)),
    ];
    for (provider, assignees) in cases {
        let lecture = Lecture {
            vault: &s,
            provider,
            id,
        };
        let err = assign_task(
            &lecture,
            &mut answer(&log, task_json("solo")),
            ids(9),
            assignees,
        )
        .unwrap_err();
        assert!(
            matches!(err, ServiceError::Invalid(_)),
            "{assignees:?}: {err:?}"
        );
    }
    assert!(
        log.borrow().is_empty(),
        "no text is sent for a refused request"
    );
    assert!(stored(&s, id).assignments.is_empty());
}

#[test]
fn a_failed_model_call_stores_no_task() {
    let (_d, s) = vault("task-down");
    let id = start(&s);
    let p = local();
    let lecture = Lecture {
        vault: &s,
        provider: &p,
        id,
    };
    let mut down = |_: &[PromptMessage]| -> Result<String, String> {
        Err("the model is not running".to_owned())
    };
    let err = assign_task(&lecture, &mut down, ids(10), Assignees::Solo(1)).unwrap_err();
    assert!(matches!(err, ServiceError::Io(_)), "{err:?}");
    assert!(stored(&s, id).assignments.is_empty());
}
