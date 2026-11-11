//! Simple chat conversation with the connected model.

use studyfrog_agent::provider::{Message, Role};

use crate::agent_types::{AgentState, ChatPurpose, selected_provider};
use crate::agent_ui::{fail, with};
use crate::effect::Effect;
use crate::state::State;

const SYSTEM_PROMPT: &str = "You are StudyFrog's study assistant. Help the user learn, explore concepts, write summaries and flashcards, and clarify difficult topics. Respond clearly and concisely.";

/// Send the current chat input as a new user message.
pub fn send_chat_message(state: State) -> (State, Vec<Effect>) {
    let input = state.agent.chat_input.trim().to_owned();
    if input.is_empty() || state.agent.chat_busy {
        return (state, Vec::new());
    }
    let Some(provider) = selected_provider(&state.agent) else {
        return fail(
            state,
            "No model selected. Choose a model in Connections first.".to_owned(),
        );
    };
    let mut messages = state.agent.chat_messages.clone();
    messages.push(Message {
        role: Role::User,
        content: input,
    });
    let (state, system) = crate::mcp_tool_calls::with_tools(state, SYSTEM_PROMPT.to_owned());
    let (state, _) = with(state, |a| AgentState {
        chat_busy: true,
        chat_input: String::new(),
        chat_messages: messages.clone(),
        ..a
    });
    (
        state,
        vec![Effect::AgentChat {
            provider,
            system,
            messages,
            purpose: ChatPurpose::Chat,
        }],
    )
}

/// The model replied to a chat message.
pub fn chat_replied(state: State, result: Result<String, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(reply) => {
            let mut messages = state.agent.chat_messages.clone();
            messages.push(Message {
                role: Role::Assistant,
                content: reply,
            });
            with(state, |a| AgentState {
                chat_busy: false,
                chat_messages: messages,
                error: None,
                ..a
            })
        }
        Err(e) => {
            let mut messages = state.agent.chat_messages.clone();
            messages.push(Message {
                role: Role::Assistant,
                content: format!("⚠️ Error: {e}"),
            });
            with(state, |a| AgentState {
                chat_busy: false,
                chat_messages: messages,
                error: Some(e),
                ..a
            })
        }
    }
}
