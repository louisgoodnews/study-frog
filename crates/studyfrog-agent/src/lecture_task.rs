//! The teacher's structured task (STORY-0210, TASK-0291): the prompt that asks the model for one solo
//! or group task as a JSON object, the parser that reads the reply, and a deterministic template used
//! when the reply does not parse. Pure: nothing here calls a model.

use serde::Deserialize;

use crate::lecture_prompt::{Line, PromptMessage, PromptRole, render_context};
use crate::rubric_eval::json_span;

/// Whether a task is for one student or for a group that hands in one answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskMode {
    /// One student works alone.
    Solo,
    /// A group of students hands in one shared answer.
    Group,
}

/// A task the teacher set, before it is attached to students.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTask {
    /// Short name shown in lists.
    pub title: String,
    /// The task text the students see.
    pub prompt: String,
    /// The grading points, in order. Never empty.
    pub rubric: Vec<String>,
    /// Solo or group.
    pub mode: TaskMode,
}

/// The most rubric points a task keeps; extra points are dropped.
pub const MAX_RUBRIC_POINTS: usize = 8;

/// The system and user messages that ask the teacher for one task of `mode`, from the material and
/// the recent conversation.
pub fn task_messages(
    teacher: &str,
    material: &[&str],
    turns: &[Line<'_>],
    mode: TaskMode,
) -> Vec<PromptMessage> {
    let system = format!(
        "You are {teacher}, the teacher in a study classroom. Set one {word} task that checks whether \
         the students understood the lecture material in the next message. {hint} Reply with one JSON \
         object and nothing else, with the keys: \"title\" (a short name), \"prompt\" (the task the \
         students see), \"mode\" (\"{word}\"), \"rubric\" (an array of 2 to 5 short grading points). \
         Use only the material.",
        word = mode_name(mode),
        hint = mode_hint(mode),
    );
    vec![
        PromptMessage {
            role: PromptRole::System,
            content: system,
        },
        PromptMessage {
            role: PromptRole::User,
            content: render_context(material, "", turns),
        },
    ]
}

/// The JSON reply as the wire carries it.
#[derive(Deserialize)]
struct Wire {
    title: String,
    prompt: String,
    mode: String,
    rubric: Vec<String>,
}

/// Reads the teacher's reply as one task of the `expected` mode. The JSON may sit inside prose. Refused
/// when there is no JSON object, the mode differs, the title or prompt is blank, or no rubric point is
/// left after trimming. The caller then uses [`fallback_task`].
pub fn parse_task(reply: &str, expected: TaskMode) -> Result<GeneratedTask, String> {
    let json = json_span(reply).ok_or("the reply holds no JSON object".to_owned())?;
    let wire: Wire = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if parse_mode(&wire.mode) != Some(expected) {
        return Err("the task is for the other mode".to_owned());
    }
    let title = wire.title.trim();
    let prompt = wire.prompt.trim();
    if title.is_empty() || prompt.is_empty() {
        return Err("the task has no title or prompt".to_owned());
    }
    let rubric: Vec<String> = wire
        .rubric
        .iter()
        .map(|point| point.trim())
        .filter(|point| !point.is_empty())
        .take(MAX_RUBRIC_POINTS)
        .map(str::to_owned)
        .collect();
    if rubric.is_empty() {
        return Err("the task has no rubric points".to_owned());
    }
    Ok(GeneratedTask {
        title: title.to_owned(),
        prompt: prompt.to_owned(),
        rubric,
        mode: expected,
    })
}

/// A fixed task about `topic`, used when the model's reply does not parse. Same input, same task.
pub fn fallback_task(topic: &str, mode: TaskMode) -> GeneratedTask {
    let topic = match topic.trim() {
        "" => "this lecture",
        named => named,
    };
    let prompt = match mode {
        TaskMode::Solo => format!(
            "In your own words, explain the main idea of {topic}. Give one example from the material."
        ),
        TaskMode::Group => format!(
            "Together, explain the main idea of {topic}. Give one example from the material and say \
             how each member's part fits in."
        ),
    };
    GeneratedTask {
        title: format!("Explain {topic}"),
        prompt,
        rubric: [
            "States the main idea correctly",
            "Gives one example from the material",
            "Explains why the idea matters",
        ]
        .iter()
        .map(|point| (*point).to_owned())
        .collect(),
        mode,
    }
}

/// The mode a name stands for, or `None` for any other word.
fn parse_mode(name: &str) -> Option<TaskMode> {
    match name.trim().to_ascii_lowercase().as_str() {
        "solo" => Some(TaskMode::Solo),
        "group" => Some(TaskMode::Group),
        _ => None,
    }
}

fn mode_name(mode: TaskMode) -> &'static str {
    match mode {
        TaskMode::Solo => "solo",
        TaskMode::Group => "group",
    }
}

fn mode_hint(mode: TaskMode) -> &'static str {
    match mode {
        TaskMode::Solo => "One student will work on it alone.",
        TaskMode::Group => "A group of students will hand in one shared answer.",
    }
}
