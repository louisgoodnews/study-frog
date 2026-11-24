//! Execution of classroom effects (TASK-0293). Each effect calls one service use case on the worker
//! thread, so a slow local model never blocks the window. The result is the session as stored now.

use studyfrog_agent::provider::Provider;
use studyfrog_core::classroom::{ClassroomSession, Participant};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::agent::Ask;
use studyfrog_service::classroom_context::Lecture;
use studyfrog_service::classroom_grade::grade_submission;
use studyfrog_service::classroom_group::group_message;
use studyfrog_service::classroom_model::{Model, provider_model};
use studyfrog_service::classroom_submit::turn_in as submit_answer;
use studyfrog_service::classroom_task::assign_task;
use studyfrog_service::classroom_turns::{
    advance_segment, simulated_question, start_session, user_message,
};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::session::{ServiceError, VaultSession};
use studyfrog_store::classroom_store::load_session;

use crate::classroom_types::{ClassroomEffect, ClassroomMsg};
use crate::exec::Worker;
use crate::msg::Msg;

/// Performs one classroom effect and reports the stored session and any failure.
pub fn run(w: &mut Worker, effect: ClassroomEffect, now_ns: i64) -> Msg {
    let now_ms = (now_ns / 1_000_000).max(0);
    let (id, outcome) = perform(w, effect, now_ms);
    let session = id.and_then(|id| reload(w, id));
    Msg::Classroom(ClassroomMsg::Done {
        session,
        error: outcome.err(),
    })
}

/// Runs the effect. Returns the session id it concerns (known before the effect runs, except for a
/// new session) and whether it worked.
fn perform(
    w: &mut Worker,
    effect: ClassroomEffect,
    now_ms: i64,
) -> (Option<Uid>, Result<(), String>) {
    let Some(vault) = w.session.as_ref() else {
        return (None, Err("Open a vault first.".to_owned()));
    };
    match effect {
        ClassroomEffect::Start {
            title,
            sources,
            participants,
            provider: _,
        } => start(vault, now_ms, &title, &sources, participants),
        ClassroomEffect::Advance { id, provider } => (
            Some(id),
            turn(w, id, &provider, now_ms, |l, m, at| {
                advance_segment(l, m, at).map(|_| ())
            }),
        ),
        ClassroomEffect::AskStudent {
            id,
            provider,
            student,
        } => (
            Some(id),
            turn(w, id, &provider, now_ms, |l, m, at| {
                simulated_question(l, m, at, student).map(|_| ())
            }),
        ),
        ClassroomEffect::Say {
            id,
            provider,
            user,
            text,
        } => (
            Some(id),
            turn(w, id, &provider, now_ms, |l, m, at| {
                user_message(l, m, at, user, &text).map(|_| ())
            }),
        ),
        ClassroomEffect::AssignTask {
            id,
            provider,
            assignees,
        } => (
            Some(id),
            turn(w, id, &provider, now_ms, |l, m, _| {
                let ids = fresh_id_source(now_ms_u64(now_ms))?;
                assign_task(l, m, ids, assignees).map(|_| ())
            }),
        ),
        ClassroomEffect::GroupMessage {
            id,
            provider,
            assignment,
            group,
            text,
        } => (
            Some(id),
            turn(w, id, &provider, now_ms, |l, m, at| {
                group_message(l, m, at, assignment, group, &text).map(|_| ())
            }),
        ),
        ClassroomEffect::TurnIn {
            id,
            provider,
            assignment,
            submitter,
            text,
        } => (
            Some(id),
            turn_in(
                &Lecture {
                    vault,
                    provider: &provider,
                    id,
                },
                now_ms,
                assignment,
                submitter,
                &text,
            ),
        ),
        ClassroomEffect::Grade {
            id,
            provider,
            assignment,
            owner,
        } => (
            Some(id),
            turn(w, id, &provider, now_ms, |l, m, at| {
                grade_submission(l, m, at, assignment, owner).map(|_| ())
            }),
        ),
    }
}

/// Stores a turned-in answer. No model is called, so this works in a build without a network.
fn turn_in(
    lecture: &Lecture<'_>,
    now_ms: i64,
    assignment: Uid,
    submitter: usize,
    text: &str,
) -> Result<(), String> {
    submit_answer(lecture, UnixMs(now_ms), assignment, submitter, text)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Creates the session. Its id is known only once it is stored.
fn start(
    vault: &VaultSession,
    now_ms: i64,
    title: &str,
    sources: &[studyfrog_core::classroom::SourceRef],
    participants: Vec<Participant>,
) -> (Option<Uid>, Result<(), String>) {
    let made = fresh_id_source(now_ms_u64(now_ms))
        .and_then(|ids| start_session(vault, ids, title, sources, participants));
    match made {
        Ok(id) => (Some(id), Ok(())),
        Err(e) => (None, Err(e.to_string())),
    }
}

/// Runs one model turn on the session `id` through `act`. The model is built from the worker's
/// transport and config folder; a build without a network or without a config folder is refused here.
fn turn(
    w: &Worker,
    id: Uid,
    provider: &Provider,
    now_ms: i64,
    act: impl FnOnce(&Lecture<'_>, &mut Model<'_>, UnixMs) -> Result<(), ServiceError>,
) -> Result<(), String> {
    let vault = w.session.as_ref().ok_or("Open a vault first.")?;
    let transport = w.transport.ok_or("This build cannot reach the network.")?;
    let config_dir = w.config_dir.clone().ok_or("there is no config folder")?;
    let log_dir = w.state_dir.clone();
    let ctx = Ask {
        config_dir: &config_dir,
        log_dir: log_dir.as_deref(),
        now_ms,
    };
    let mut model = provider_model(transport, provider, ctx);
    let lecture = Lecture {
        vault,
        provider,
        id,
    };
    act(&lecture, &mut model, UnixMs(now_ms)).map_err(|e| e.to_string())
}

fn reload(w: &Worker, id: Uid) -> Option<ClassroomSession> {
    let vault = w.session.as_ref()?;
    load_session(&vault.conn, id).ok().flatten()
}

fn now_ms_u64(now_ms: i64) -> u64 {
    u64::try_from(now_ms).unwrap_or(0)
}
