//! The model behind a classroom turn (STORY-0209, TASK-0290): the existing [`crate::agent::ask`] path,
//! reached through a closure that takes prompt messages and returns the reply. Local and remote
//! providers both go through it, so no new client or wire format is needed here.

use studyfrog_agent::lecture_prompt::{PromptMessage, PromptRole};
use studyfrog_agent::provider::{Message, Provider, Role, Transport, may_send};

use crate::agent::{Ask, ask};
use crate::session::ServiceError;

/// One model turn: the prompt messages in, the reply text out. Tests pass a fake closure; the
/// application passes the closure from [`provider_model`].
pub type Model<'a> = dyn FnMut(&[PromptMessage]) -> Result<String, String> + 'a;

/// Refuses before anything is sent when `provider` is remote and the user has not allowed vault text
/// to go to it. Local providers are always allowed.
pub fn check_consent(provider: &Provider) -> Result<(), ServiceError> {
    if may_send(provider) {
        Ok(())
    } else {
        Err(ServiceError::Invalid(format!(
            "Text from your vault may not be sent to {} until you allow it for that connection.",
            provider.name
        )))
    }
}

/// A [`Model`] that sends each turn to `provider` through [`ask`]. The system messages become the
/// system text; the user messages become the conversation.
pub fn provider_model<'a>(
    transport: Transport,
    provider: &'a Provider,
    ctx: Ask<'a>,
) -> impl FnMut(&[PromptMessage]) -> Result<String, String> + 'a {
    move |messages| {
        let system = messages
            .iter()
            .filter(|m| m.role == PromptRole::System)
            .map(|m| m.content.as_str())
            .collect::<Vec<&str>>()
            .join("\n\n");
        let conversation: Vec<Message> = messages
            .iter()
            .filter(|m| m.role == PromptRole::User)
            .map(|m| Message {
                role: Role::User,
                content: m.content.clone(),
            })
            .collect();
        ask(transport, provider, ctx, &system, &conversation)
    }
}
