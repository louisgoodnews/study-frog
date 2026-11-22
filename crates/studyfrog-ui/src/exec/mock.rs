//! Making and saving a mock exam from notes (STORY-0186): the notes are read from the vault, the model (if
//! one may see them) writes the questions, and the exam is saved as a note.

use studyfrog_agent::provider::{Message, Role};
use studyfrog_service::agent::{Ask, ask};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::exam_gen::generate;
use studyfrog_service::exam_save::{notes_chunks, save_exam};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::setup::mock_exam::{MockExamEffect, MockExamMsg};

/// Perform a mock-exam effect and answer with its result.
pub fn run(w: &mut Worker, effect: MockExamEffect, now_ns: i64) -> Msg {
    match effect {
        MockExamEffect::Generate {
            folder,
            total,
            title,
            provider,
        } => generate_exam(w, folder, total, &title, provider, now_ns),
        MockExamEffect::Save {
            folder,
            title,
            exam,
            keys,
        } => {
            let done = match (
                w.session.as_mut(),
                fresh_id_source((now_ns / 1_000_000) as u64),
            ) {
                (None, _) => Err("no vault is open".to_owned()),
                (_, Err(e)) => Err(studyfrog_service::session::describe_service_error(&e)),
                (Some(s), Ok(_)) => save_exam(
                    s,
                    folder.as_deref(),
                    &title,
                    &exam,
                    &keys,
                    (now_ns / 1_000_000) as u64,
                )
                .map_err(|e| studyfrog_service::session::describe_service_error(&e)),
            };
            Msg::MockExam(MockExamMsg::Saved(done))
        }
    }
}

fn generate_exam(
    w: &Worker,
    folder: Option<String>,
    total: usize,
    title: &str,
    provider: Option<studyfrog_agent::provider::Provider>,
    now_ns: i64,
) -> Msg {
    let Some(session) = w.session.as_ref() else {
        return Msg::MockExam(MockExamMsg::Generated(Err("no vault is open".to_owned())));
    };
    let chunks = match notes_chunks(session, folder.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            return Msg::MockExam(MockExamMsg::Generated(Err(
                studyfrog_service::session::describe_service_error(&e),
            )));
        }
    };
    let now_ms = now_ns / 1_000_000;
    let generated = match (provider, w.transport, w.config_dir.clone()) {
        (Some(p), Some(transport), Some(config_dir)) => {
            let log = w.state_dir.clone();
            let mut ask_model = |system: &str, user: &str| {
                let ctx = Ask {
                    config_dir: &config_dir,
                    log_dir: log.as_deref(),
                    now_ms,
                };
                let messages = [Message {
                    role: Role::User,
                    content: user.to_owned(),
                }];
                ask(transport, &p, ctx, system, &messages)
            };
            generate(&chunks, total, title, Some(&mut ask_model))
        }
        _ => generate(&chunks, total, title, None),
    };
    Msg::MockExam(MockExamMsg::Generated(Ok(Box::new(generated))))
}
