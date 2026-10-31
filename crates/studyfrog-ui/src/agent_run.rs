//! Sending a wiki request, reading its answer and running an exam: the steps of the assistant screen
//! that talk to a model. Pure.

use studyfrog_agent::exam::{exam_system, parse_results, without_results};
use studyfrog_agent::provider::{Message, Role, is_local};
use studyfrog_agent::wiki::{wiki_system, wiki_user};

use crate::agent_types::{
    AgentState, ChatPurpose, ExamState, Pending, Reviewing, selected_provider,
};
use crate::agent_ui::{fail, need_provider, with};
use crate::effect::Effect;
use crate::state::State;

pub(crate) fn prepared(
    state: State,
    ctx: Result<Box<studyfrog_service::agent_vault::WikiContext>, String>,
) -> (State, Vec<Effect>) {
    let ctx = match ctx {
        Ok(c) => c,
        Err(e) => return fail(state, e),
    };
    let pending = Pending {
        stacks: ctx.stacks,
        items: ctx.items,
        chars: ctx.chars,
        left_out: ctx.left_out,
    };
    let local = selected_provider(&state.agent).is_some_and(|p| is_local(&p));
    let (state, _) = with(state, |a| AgentState {
        pending: Some(pending),
        error: None,
        ..a
    });
    if local {
        send_wiki(state)
    } else {
        (state, Vec::new())
    }
}

pub(crate) fn send_wiki(state: State) -> (State, Vec<Effect>) {
    let (Some(p), Some(pending)) = (selected_provider(&state.agent), state.agent.pending.clone())
    else {
        return (state, Vec::new());
    };
    let system = wiki_system();
    let user = wiki_user(&state.agent.instruction, &pending.stacks, &pending.items);
    let messages = vec![Message {
        role: Role::User,
        content: user,
    }];
    let (state, _) = with(state, |a| AgentState {
        busy: true,
        pending: None,
        ..a
    });
    (
        state,
        vec![Effect::AgentChat {
            provider: p,
            system,
            messages,
            purpose: ChatPurpose::Wiki(
                pending.items.iter().map(|i| i.uid.clone()).collect(),
                pending.stacks.iter().map(|s| s.0.clone()).collect(),
            ),
        }],
    )
}

pub(crate) fn wiki_replied(
    state: State,
    result: Result<Box<studyfrog_agent::wiki::Proposal>, String>,
) -> (State, Vec<Effect>) {
    match result {
        Err(e) => fail(state, e),
        Ok(p) => {
            let accepted = vec![true; p.changes.len()];
            with(state, |a| AgentState {
                busy: false,
                error: None,
                applied: None,
                reviewing: Some(Reviewing {
                    proposal: *p,
                    accepted,
                }),
                ..a
            })
        }
    }
}

pub(crate) fn start_exam(state: State) -> (State, Vec<Effect>) {
    let count = state
        .agent
        .exam_count
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|n| (1..=50).contains(n));
    let Some(count) = count else {
        return fail(
            state,
            "The number of questions is between 1 and 50.".to_owned(),
        );
    };
    match need_provider(&state.agent) {
        Err(e) => fail(state, e),
        Ok(_) => {
            let folders = state.agent.folders.clone();
            let (state, _) = with(state, |a| AgentState {
                busy: true,
                error: None,
                exam: ExamState::default(),
                ..a
            });
            (state, vec![Effect::PrepareExam { folders, count }])
        }
    }
}

pub(crate) fn chat(state: State, messages: Vec<Message>) -> (State, Vec<Effect>) {
    let Some(p) = selected_provider(&state.agent) else {
        return (state, Vec::new());
    };
    let system = exam_system(&state.agent.exam.items);
    let (state, _) = with(state, |a| AgentState {
        busy: true,
        exam: ExamState {
            chat: messages.clone(),
            ..a.exam
        },
        ..a
    });
    (
        state,
        vec![Effect::AgentChat {
            provider: p,
            system,
            messages,
            purpose: ChatPurpose::Exam,
        }],
    )
}

pub(crate) fn exam_replied(state: State, result: Result<String, String>) -> (State, Vec<Effect>) {
    let text = match result {
        Ok(t) => t,
        Err(e) => return fail(state, e),
    };
    let uids: Vec<String> = state
        .agent
        .exam
        .items
        .iter()
        .map(|i| i.uid.clone())
        .collect();
    let results = parse_results(&text, &uids);
    let shown = Message {
        role: Role::Assistant,
        content: text.clone(),
    };
    with(state, |a| {
        let mut chat = a.exam.chat.clone();
        chat.push(Message {
            content: without_results(&shown.content),
            ..shown
        });
        AgentState {
            busy: false,
            error: None,
            exam: ExamState {
                chat,
                results: results.or(a.exam.results.clone()),
                ..a.exam
            },
            ..a
        }
    })
}
