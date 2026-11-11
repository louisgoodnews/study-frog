//! The assistant screen's update logic: connections, wiki proposals and exams. Pure.

use studyfrog_agent::exam::exam_opening;
use studyfrog_agent::provider::{Message, Provider, Role, may_send};
use studyfrog_agent::wiki::parse_proposal;

use crate::agent_mcp_types::AgentEffect;
pub use crate::agent_types::*;
use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

pub(crate) fn with(state: State, f: impl FnOnce(AgentState) -> AgentState) -> (State, Vec<Effect>) {
    let agent = f(state.agent.clone());
    (State { agent, ..state }, Vec::new())
}

pub(crate) fn fail(state: State, why: String) -> (State, Vec<Effect>) {
    let agent = AgentState {
        busy: false,
        error: Some(why),
        ..state.agent.clone()
    };
    (State { agent, ..state }, Vec::new())
}

pub(crate) fn notice(state: State, text: String) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Info,
        text,
    });
    (State { notice, ..state }, Vec::new())
}

pub(crate) fn need_provider(a: &AgentState) -> Result<Provider, String> {
    let p = selected_provider(a).ok_or("Choose a connection first (the Connections tab).")?;
    if a.folders.is_empty() {
        return Err("Tick the stacks the assistant may read.".to_owned());
    }
    if !may_send(&p) {
        return Err(format!(
            "Allow sending text to {} on the Connections tab first.",
            p.name
        ));
    }
    Ok(p)
}

/// Handle the assistant's messages.
pub fn update(state: State, msg: AgentMsg) -> (State, Vec<Effect>) {
    match msg {
        AgentMsg::Loaded(c) => crate::agent_conn::loaded(state, c),
        AgentMsg::Saved(Ok(c)) => {
            let (state, effects) = crate::agent_conn::loaded(state, c);
            let agent = AgentState {
                form: AgentState::default().form,
                error: None,
                ..state.agent.clone()
            };
            (State { agent, ..state }, effects)
        }
        AgentMsg::Saved(Err(e)) => fail(state, e),
        AgentMsg::Tab(tab) => with(state, |a| AgentState { tab, ..a }),
        AgentMsg::Select(n) => with(state, |a| AgentState {
            selected: Some(n),
            pending: None,
            ..a
        }),
        AgentMsg::Folder(f, on) => with(state, |a| {
            let mut folders: Vec<String> = a.folders.iter().filter(|x| **x != f).cloned().collect();
            if on {
                folders.push(f);
            }
            AgentState { folders, ..a }
        }),
        AgentMsg::Form(e) => with(state, |a| crate::agent_conn::form_edit(a, e)),
        AgentMsg::AddConnection => crate::agent_conn::add_connection(state),
        AgentMsg::Test(m) => crate::agent_conn_test::update(state, m),
        AgentMsg::RemoveConnection(n) => {
            (state, vec![Effect::Agent(AgentEffect::RemoveConnection(n))])
        }
        AgentMsg::Allow(n, on) => (
            state,
            vec![Effect::Agent(AgentEffect::AllowConnection(n, on))],
        ),
        AgentMsg::Mcp(m) => crate::mcp_ui::update(state, m),
        AgentMsg::Instruction(t) => with(state, |a| AgentState {
            instruction: t,
            ..a
        }),
        AgentMsg::UsePdfPages(path, first, last) => with(state, |a| AgentState {
            pdf_pages: Some((path, first, last)),
            tab: crate::agent_types::AgentTab::Wiki,
            ..a
        }),
        AgentMsg::UseHelpPages(on) => with(state, |a| AgentState {
            help_pages: on,
            tab: crate::agent_types::AgentTab::Wiki,
            ..a
        }),
        AgentMsg::ClearPdfPages => with(state, |a| AgentState {
            pdf_pages: None,
            ..a
        }),
        AgentMsg::PrepareWiki => match need_provider(&state.agent) {
            Err(e) => fail(state, e),
            Ok(_) => {
                let folders = state.agent.folders.clone();
                let pdf = state.agent.pdf_pages.clone();
                let (state, _) = with(state, |a| AgentState {
                    busy: true,
                    error: None,
                    ..a
                });
                (state, vec![Effect::PrepareWiki { folders, pdf }])
            }
        },
        AgentMsg::WikiPrepared(r) => {
            let (state, _) = with(state, |a| AgentState { busy: false, ..a });
            crate::agent_run::prepared(state, r)
        }
        AgentMsg::SendWiki => crate::agent_run::send_wiki(state),
        AgentMsg::WikiReplied(r) => crate::agent_run::wiki_replied(state, r),
        AgentMsg::Accept(i, on) => with(state, |a| {
            let reviewing = a.reviewing.clone().map(|mut r| {
                if let Some(slot) = r.accepted.get_mut(i) {
                    *slot = on;
                }
                r
            });
            AgentState { reviewing, ..a }
        }),
        AgentMsg::ApplyProposal => match state
            .agent
            .reviewing
            .as_ref()
            .map(accepted_changes)
            .filter(|c| !c.is_empty())
        {
            Some(changes) => {
                let (state, _) = with(state, |a| AgentState { busy: true, ..a });
                (state, vec![Effect::ApplyChanges(changes)])
            }
            None => fail(state, "Accept at least one change first.".to_owned()),
        },
        AgentMsg::ProposalApplied(Err(e)) => fail(state, e),
        AgentMsg::ProposalApplied(Ok(done)) => crate::mcp_ui::applied(state, *done),
        AgentMsg::Discard => with(state, |a| AgentState {
            pending: None,
            reviewing: None,
            error: None,
            ..a
        }),
        AgentMsg::ExamCount(t) => with(state, |a| AgentState { exam_count: t, ..a }),
        AgentMsg::StartExam => crate::agent_run::start_exam(state),
        AgentMsg::ExamPrepared(Err(e)) => fail(state, e),
        AgentMsg::ExamPrepared(Ok(items)) if items.is_empty() => fail(
            state,
            "The chosen stacks have no cards or questions to ask.".to_owned(),
        ),
        AgentMsg::ExamPrepared(Ok(items)) => {
            let (state, _) = with(state, |a| AgentState {
                exam: ExamState {
                    items,
                    ..ExamState::default()
                },
                ..a
            });
            crate::agent_run::chat(
                state,
                vec![Message {
                    role: Role::User,
                    content: exam_opening(),
                }],
            )
        }
        AgentMsg::ExamInput(t) => with(state, |a| AgentState {
            exam: ExamState { input: t, ..a.exam },
            ..a
        }),
        AgentMsg::ExamSend => {
            let text = state.agent.exam.input.trim().to_owned();
            if text.is_empty() || state.agent.busy {
                return (state, Vec::new());
            }
            let mut messages = state.agent.exam.chat.clone();
            messages.push(Message {
                role: Role::User,
                content: text,
            });
            let (state, _) = with(state, |a| AgentState {
                exam: ExamState {
                    input: String::new(),
                    ..a.exam
                },
                ..a
            });
            crate::agent_run::chat(state, messages)
        }
        AgentMsg::ExamReplied(r) => crate::agent_run::exam_replied(state, r),
        AgentMsg::RecordExam => match state.agent.exam.results.clone() {
            Some(r) if state.agent.exam.recorded.is_none() => (state, vec![Effect::RecordExam(r)]),
            _ => (state, Vec::new()),
        },
        AgentMsg::ExamRecorded(Err(e)) => fail(state, e),
        AgentMsg::ExamRecorded(Ok(n)) => with(state, |a| AgentState {
            exam: ExamState {
                recorded: Some(n),
                ..a.exam
            },
            ..a
        }),
        AgentMsg::ChatInput(text) => with(state, |a| AgentState {
            chat_input: text,
            ..a
        }),
        AgentMsg::ChatSend => crate::agent_chat::send_chat_message(state),
        AgentMsg::ChatReplied(r) => crate::agent_chat::chat_replied(state, r),
        AgentMsg::ChatClear => with(state, |a| AgentState {
            chat_messages: Vec::new(),
            ..a
        }),
    }
}

/// The text of a proposal parsed from a model's reply (used by the worker).
pub fn read_proposal(
    reply: &str,
    uids: &[String],
    stacks: &[String],
) -> Result<Box<studyfrog_agent::wiki::Proposal>, String> {
    parse_proposal(reply, uids, stacks).map(Box::new)
}
