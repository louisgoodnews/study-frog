//! Making, grading and saving a mock exam (STORY-0186, STORY-0187, STORY-0207).

use studyfrog_agent::provider::{Message, Provider, Role};
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::exam_answer::ExamAnswer;
use studyfrog_rehearsal::exam_format::ExamFormatMix;
use studyfrog_rehearsal::mock_exam::MockExam;
use studyfrog_service::agent::{Ask, ask};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::exam_gen::generate_with_mix;
use studyfrog_service::exam_save::{notes_chunks, save_exam, slide_chunks};
use studyfrog_service::pdfs::{index_pdfs_fully, pdf_library};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::setup::mock_exam::grading::grade_with;
use crate::setup::mock_exam::{ExamSource, MockExamEffect, MockExamMsg, SlideChoice};

pub fn run(w: &mut Worker, effect: MockExamEffect, now_ns: i64) -> Msg {
    match effect {
        MockExamEffect::Generate {
            source,
            mix,
            title,
            provider,
        } => generate_exam(w, &source, &mix, &title, provider.as_ref(), now_ns),
        MockExamEffect::ListSlides => list_slides(w, now_ns),
        MockExamEffect::Grade {
            exam,
            keys,
            answers,
            provider,
        } => grade_exam(w, &exam, &keys, &answers, &provider, now_ns),
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

fn list_slides(w: &mut Worker, now_ns: i64) -> Msg {
    let result = match w.session.as_mut() {
        None => Err("no vault is open".to_owned()),
        Some(session) => index_pdfs_fully(session, now_ns)
            .and_then(|()| pdf_library(session))
            .map(|rows| {
                rows.into_iter()
                    .filter(|r| r.error.is_none())
                    .map(|r| SlideChoice {
                        title: crate::pdf_names::label(&r.path, &r.title),
                        path: r.path,
                    })
                    .collect()
            })
            .map_err(|e| studyfrog_service::session::describe_service_error(&e)),
    };
    Msg::MockExam(MockExamMsg::Slides(result))
}

fn generate_exam(
    w: &Worker,
    source: &ExamSource,
    mix: &ExamFormatMix,
    title: &str,
    provider: Option<&Provider>,
    now_ns: i64,
) -> Msg {
    let Some(session) = w.session.as_ref() else {
        return Msg::MockExam(MockExamMsg::Generated(Err("no vault is open".to_owned())));
    };
    let read = match source {
        ExamSource::Notes(folder) => notes_chunks(session, folder.as_deref()),
        ExamSource::Slide(path) => slide_chunks(session, path),
    };
    let chunks = match read {
        Ok(c) => c,
        Err(e) => {
            return Msg::MockExam(MockExamMsg::Generated(Err(
                studyfrog_service::session::describe_service_error(&e),
            )));
        }
    };
    let now_ms = now_ns / 1_000_000;
    let outcome = match provider {
        Some(p) => {
            let mut ask_model = |system: &str, user: &str| model_call(w, p, now_ms, system, user);
            generate_with_mix(&chunks, mix, title, Some(&mut ask_model))
        }
        None => generate_with_mix(&chunks, mix, title, None),
    };
    Msg::MockExam(MockExamMsg::Generated(outcome.map(Box::new)))
}

fn grade_exam(
    w: &Worker,
    exam: &MockExam,
    keys: &[AnswerKey],
    answers: &[ExamAnswer],
    provider: &Provider,
    now_ns: i64,
) -> Msg {
    let now_ms = now_ns / 1_000_000;
    let mut ask_model = |system: &str, user: &str| model_call(w, provider, now_ms, system, user);
    let graded = grade_with(exam, keys, answers, Some(&mut ask_model));
    Msg::MockExam(MockExamMsg::Graded(graded))
}

fn model_call(
    w: &Worker,
    p: &Provider,
    now_ms: i64,
    system: &str,
    user: &str,
) -> Result<String, String> {
    let (Some(transport), Some(config_dir)) = (w.transport, w.config_dir.clone()) else {
        return Err("the model cannot be reached from this build".to_owned());
    };
    let log = w.state_dir.clone();
    let ctx = Ask {
        config_dir: &config_dir,
        log_dir: log.as_deref(),
        now_ms,
    };
    let messages = [Message {
        role: Role::User,
        content: user.to_owned(),
    }];
    ask(transport, p, ctx, system, &messages)
}
