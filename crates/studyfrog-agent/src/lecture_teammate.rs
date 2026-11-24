//! Prompt builder for a simulated teammate's contribution to a group task (STORY-0210, TASK-0291).
//! The teammate adds one idea or sub-answer to the group thread and never writes the group's final
//! answer. Pure.

use crate::lecture_prompt::{Line, PromptMessage, PromptRole};
use crate::lecture_student::Persona;

/// The system and user messages for one contribution. `thread` is the group's own conversation so far,
/// oldest first.
pub fn teammate_messages(
    persona: &Persona<'_>,
    task_title: &str,
    task_prompt: &str,
    thread: &[Line<'_>],
) -> Vec<PromptMessage> {
    let system = format!(
        "You are {}, a student in a study group. You are {}. Your group has the task \"{task_title}\": \
         {task_prompt} Add one contribution to the group thread: an idea, a sub-answer, or a reply to a \
         teammate. Write in your own words, in one to three sentences. Do not write the group's final \
         answer and do not speak for the other students.",
        persona.name, persona.stance
    );
    let body: Vec<String> = thread
        .iter()
        .map(|line| format!("{}: {}", line.speaker, line.text))
        .collect();
    let thread_text = if body.is_empty() {
        "(none yet)".to_owned()
    } else {
        body.join("\n")
    };
    vec![
        PromptMessage {
            role: PromptRole::System,
            content: system,
        },
        PromptMessage {
            role: PromptRole::User,
            content: format!("Group thread so far:\n{thread_text}"),
        },
    ]
}
