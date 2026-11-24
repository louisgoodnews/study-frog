#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers shared by the classroom turn-in and grading tests (TASK-0292): the teacher's task reply, a fake
//! model, a session with a solo task and answer, and the error check. Built on `classroom_common`.

use std::cell::RefCell;

use studyfrog_agent::lecture_prompt::PromptMessage;
use studyfrog_core::classroom::Participant;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_submit::turn_in;
use studyfrog_service::classroom_task::{Assignees, assign_task};
use studyfrog_service::classroom_turns::start_session;
use studyfrog_service::create::IdSource;
use studyfrog_service::session::{ServiceError, VaultSession};

use crate::classroom_common::*;

pub fn task_reply(mode: &str) -> String {
    format!(
        r#"{{"title": "Cell walls", "prompt": "Say what a cell wall does.", "mode": "{mode}", "rubric": ["Names the function", "Gives an example"]}}"#
    )
}

pub fn ids(random: u8) -> IdSource {
    IdSource {
        now_ms: 1_000,
        random: [random; 10],
    }
}

/// A fake model that logs each prompt's system text and always gives `reply`.
pub fn reply_with<'a>(
    log: &'a RefCell<Vec<String>>,
    reply: Result<String, String>,
) -> impl FnMut(&[PromptMessage]) -> Result<String, String> + 'a {
    move |messages| {
        log.borrow_mut().push(messages[0].content.clone());
        reply.clone()
    }
}

/// Sets a task of `mode` for `assignees` through the teacher, with a model that gives the task JSON.
pub fn set_task(lecture: &Lecture<'_>, mode: &str, assignees: Assignees, random: u8) -> Uid {
    let log = RefCell::new(Vec::new());
    assign_task(
        lecture,
        &mut reply_with(&log, Ok(task_reply(mode))),
        ids(random),
        assignees,
    )
    .unwrap()
}

/// A session of `people`, grounded in the usual sources, on the vault; its id.
pub fn session_of(s: &VaultSession, people: Vec<Participant>) -> Uid {
    let ids = IdSource {
        now_ms: 1_000,
        random: [7; 10],
    };
    start_session(s, ids, "Cell lecture", &sources(), people).unwrap()
}

pub fn refused_with(err: ServiceError, word: &str) -> bool {
    matches!(err, ServiceError::Invalid(msg) if msg.contains(word))
}

pub const GRADED_AT: UnixMs = UnixMs(8_000);

/// The rubric is "Names the function" and "Gives an example" (see `task_reply`).
pub const ANSWER: &str = "It names the function only.";

/// The teacher's grade in which the model finds both rubric points met.
pub const MODEL_BOTH_MET: &str = r#"{"criteria": [{"criterion": "Names the function", "met": true, "feedback": "Clear."}, {"criterion": "Gives an example", "met": true, "feedback": "A good example."}], "summary": "Well argued overall."}"#;

/// A session with a solo task for Ada (participant 1) and Ada's answer turned in, unless `turn_in_answer`
/// is false. Returns the session id and the task id.
pub fn solo_task_with_answer(s: &VaultSession, turn_in_answer: bool) -> (Uid, Uid) {
    let id = start(s);
    let p = local();
    let lecture = Lecture {
        vault: s,
        provider: &p,
        id,
    };
    let log = RefCell::new(Vec::new());
    let task = assign_task(
        &lecture,
        &mut reply_with(&log, Ok(task_reply("solo"))),
        ids(3),
        Assignees::Solo(1),
    )
    .unwrap();
    if turn_in_answer {
        turn_in(&lecture, AT, task, 1, ANSWER).unwrap();
    }
    (id, task)
}
