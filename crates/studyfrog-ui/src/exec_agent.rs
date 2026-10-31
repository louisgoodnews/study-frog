//! Performing the assistant's effects: connections, questions to a model, proposals and exams.

use std::path::{Path, PathBuf};

use studyfrog_agent::exam::ExamResult;
use studyfrog_agent::provider::{Message, Provider};
use studyfrog_agent::wiki::Change;
use studyfrog_core::time::UnixMs;
use studyfrog_service::agent::{Ask, ask, load_connections, save_providers, set_key};
use studyfrog_service::agent_pdf::wiki_context_with_pdf;
use studyfrog_service::agent_vault::{apply_changes, exam_items, record_exam};
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::session::vault_info;

use crate::agent_types::{AgentMsg, ChatPurpose};
use crate::effect::Effect;
use crate::exec::Worker;
use crate::msg::Msg;

fn agent(m: AgentMsg) -> Msg {
    Msg::Agent(m)
}

fn dir(w: &Worker) -> Option<PathBuf> {
    w.config_dir.clone()
}

/// Read the connections of the profile in use.
pub fn load(w: &Worker) -> Msg {
    agent(AgentMsg::Loaded(
        dir(w).map_or_else(Default::default, |d| load_connections(&d)),
    ))
}

fn change_connections(
    w: &Worker,
    f: impl FnOnce(&Path, Vec<Provider>) -> Result<(), String>,
) -> Msg {
    let Some(d) = dir(w) else {
        return agent(AgentMsg::Saved(Err("there is no config folder".to_owned())));
    };
    let current = load_connections(&d);
    let result = f(&d, current.providers).map(|()| load_connections(&d));
    agent(AgentMsg::Saved(result))
}

/// Save a connection and its key.
pub fn save(w: &Worker, provider: Provider, key: Option<String>) -> Msg {
    change_connections(w, |d, mut list| {
        list.push(provider.clone());
        save_providers(d, &list).map_err(|e| e.to_string())?;
        set_key(d, &provider.name, key.as_deref()).map_err(|e| e.to_string())
    })
}

/// Remove a connection and its key.
pub fn remove(w: &Worker, name: &str) -> Msg {
    change_connections(w, |d, list| {
        let rest: Vec<Provider> = list.into_iter().filter(|p| p.name != name).collect();
        save_providers(d, &rest).map_err(|e| e.to_string())?;
        set_key(d, name, None).map_err(|e| e.to_string())
    })
}

/// Allow or forbid sending text to a connection.
pub fn allow(w: &Worker, name: &str, on: bool) -> Msg {
    change_connections(w, |d, list| {
        let changed: Vec<Provider> = list
            .into_iter()
            .map(|p| {
                if p.name == name {
                    Provider { allowed: on, ..p }
                } else {
                    p
                }
            })
            .collect();
        save_providers(d, &changed).map_err(|e| e.to_string())
    })
}

/// What a wiki request over these stacks would send.
pub fn prepare_wiki(w: &Worker, folders: &[String], pdf: Option<&(String, usize, usize)>) -> Msg {
    let result = match &w.session {
        Some(s) => wiki_context_with_pdf(s, folders, pdf.map(|(p, a, b)| (p.as_str(), *a, *b)))
            .map(Box::new)
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    agent(AgentMsg::WikiPrepared(result))
}

/// One question to a model; the answer is read according to its purpose.
pub fn chat(
    w: &Worker,
    (provider, system, messages): (Provider, String, Vec<Message>),
    purpose: ChatPurpose,
    now_ns: i64,
) -> Msg {
    let reply = match (w.transport, dir(w)) {
        (Some(transport), Some(config_dir)) => {
            let log = w.state_dir.clone();
            let ctx = Ask {
                config_dir: &config_dir,
                log_dir: log.as_deref(),
                now_ms: now_ns / 1_000_000,
            };
            ask(transport, &provider, ctx, &system, &messages)
        }
        (None, _) => Err("This build cannot reach the network.".to_owned()),
        (_, None) => Err("there is no config folder".to_owned()),
    };
    match purpose {
        ChatPurpose::Exam => agent(AgentMsg::ExamReplied(reply)),
        ChatPurpose::Wiki(uids, stacks) => {
            agent(AgentMsg::WikiReplied(reply.and_then(|t| {
                crate::agent_ui::read_proposal(&t, &uids, &stacks)
            })))
        }
    }
}

/// Apply the accepted changes and report the refreshed vault.
pub fn apply(w: &mut Worker, changes: &[Change], now_ns: i64) -> Msg {
    let now_ms = now_ns / 1_000_000;
    let result = match &mut w.session {
        Some(s) => fresh_id_source(u64::try_from(now_ms).unwrap_or(0)).and_then(|ids| {
            let done = apply_changes(s, changes, ids, UnixMs(now_ms));
            vault_info(&s.root, &s.conn).map(|info| Box::new((done, info)))
        }),
        None => Err(studyfrog_service::session::ServiceError::Invalid(
            "no vault is open".to_owned(),
        )),
    };
    agent(AgentMsg::ProposalApplied(result.map_err(|e| e.to_string())))
}

/// Choose the questions of an exam.
pub fn prepare_exam(w: &Worker, folders: &[String], count: usize) -> Msg {
    let result = match &w.session {
        Some(s) => exam_items(s, folders, count).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    agent(AgentMsg::ExamPrepared(result))
}

/// Record exam verdicts as reviews.
pub fn record(w: &mut Worker, results: &[ExamResult], now_ns: i64) -> Msg {
    let result = match &mut w.session {
        Some(s) => record_exam(s, results, UnixMs(now_ns / 1_000_000)).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    agent(AgentMsg::ExamRecorded(result))
}

/// Perform one of the assistant's effects.
pub fn run(w: &mut Worker, effect: Effect, now_ns: i64) -> Msg {
    match effect {
        Effect::LoadAgent => load(w),
        Effect::SaveConnection { provider, key } => save(w, provider, key),
        Effect::RemoveConnection(name) => remove(w, &name),
        Effect::AllowConnection(name, on) => allow(w, &name, on),
        Effect::PrepareWiki { folders, pdf } => prepare_wiki(w, &folders, pdf.as_ref()),
        Effect::AgentChat {
            provider,
            system,
            messages,
            purpose,
        } => chat(w, (provider, system, messages), purpose, now_ns),
        Effect::ApplyChanges(changes) => apply(w, &changes, now_ns),
        Effect::PrepareExam { folders, count } => prepare_exam(w, &folders, count),
        Effect::RecordExam(results) => record(w, &results, now_ns),
        other => agent(AgentMsg::Saved(Err(format!(
            "not an assistant effect: {other:?}"
        )))),
    }
}
