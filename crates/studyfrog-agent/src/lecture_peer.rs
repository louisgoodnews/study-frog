//! Prompt builder for the answer a simulated group hands in for a group task (STORY-0210, TASK-0292).
//! The answer is written for comparison with the user's group's answer. Pure: nothing here calls a model.

use crate::lecture_prompt::{PromptMessage, PromptRole};

/// The system and user messages that ask a simulated group for its final answer to the task. `members`
/// are the group's student names, in roster order.
pub fn peer_messages(
    task_title: &str,
    task_prompt: &str,
    rubric: &[String],
    members: &[&str],
) -> Vec<PromptMessage> {
    let points: Vec<String> = rubric.iter().map(|point| format!("- {point}")).collect();
    let system = format!(
        "You are a study group made of {names}. Your group has the task \"{task_title}\": {task_prompt}\n\n\
         Rubric points:\n{points}\n\nWrite the group's one final answer in two to four sentences, in your \
         own words, so that it makes the rubric points. Reply with the answer only.",
        names = members.join(" and "),
        points = points.join("\n"),
    );
    vec![
        PromptMessage {
            role: PromptRole::System,
            content: system,
        },
        PromptMessage {
            role: PromptRole::User,
            content: "Write the group's final answer now.".to_owned(),
        },
    ]
}
