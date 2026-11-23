//! Prompt builders for the AI teacher and for rolling lecture summaries (STORY-0209, TASK-0289).
//! Pure: text in, messages out. Nothing here calls a model, reads a clock or draws randomness.
//! Each builder returns exactly one system message (the persona and task) and one user message
//! (the material and conversation). The system text never carries another persona's instructions.

/// Who a prompt message is addressed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptRole {
    /// Instructions that frame the persona and the task.
    System,
    /// The material and the conversation the model reacts to.
    User,
}

/// One message ready for a chat request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptMessage {
    /// Who the message is addressed to.
    pub role: PromptRole,
    /// The text.
    pub content: String,
}

/// One line of the lecture chat: who said it and what was said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line<'a> {
    /// Display name of the speaker.
    pub speaker: &'a str,
    /// What was said.
    pub text: &'a str,
}

/// What the teacher is asked to do on this turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeacherMove {
    /// Explain the next segment of the lecture, building on what was said so far.
    NextSegment,
    /// Answer the latest question in the conversation from the material.
    AnswerQuestion,
    /// Ask one short question that checks whether the class followed the last segment.
    CheckComprehension,
}

/// The system and user messages for one teacher turn.
pub fn teacher_messages(
    teacher: &str,
    material: &[&str],
    summary: &str,
    turns: &[Line<'_>],
    step: TeacherMove,
) -> Vec<PromptMessage> {
    vec![
        message(PromptRole::System, teacher_system(teacher, step)),
        message(PromptRole::User, render_context(material, summary, turns)),
    ]
}

/// The system and user messages that fold new lines into the rolling lecture summary. The reply is
/// the new summary and must stay within `max_words` words.
pub fn summary_messages(
    material: &[&str],
    previous: &str,
    new_turns: &[Line<'_>],
    max_words: usize,
) -> Vec<PromptMessage> {
    let system = format!(
        "You keep the running summary of a lecture. Rewrite the summary so it covers the previous \
         summary and the new lines of conversation. Keep only what a student needs to follow the \
         lecture next: the concepts taught, the definitions given and the questions still open. \
         Use the lecture material for the facts. Answer with the summary alone, in at most {max_words} \
         words."
    );
    let mut user = String::from("Lecture material:\n");
    push_material(&mut user, material);
    user.push_str("\n\nPrevious summary:\n");
    user.push_str(or_none(previous));
    user.push_str("\n\nNew lines:\n");
    push_lines(&mut user, new_turns);
    vec![
        message(PromptRole::System, system),
        message(PromptRole::User, user),
    ]
}

fn teacher_system(teacher: &str, step: TeacherMove) -> String {
    let task = match step {
        TeacherMove::NextSegment => {
            "Task: explain the next segment of the lecture, building on the summary and the recent conversation."
        }
        TeacherMove::AnswerQuestion => {
            "Task: answer the latest question in the conversation, using the material."
        }
        TeacherMove::CheckComprehension => {
            "Task: ask one short question that checks whether the class understood the last segment."
        }
    };
    format!(
        "You are {teacher}, the teacher in a study classroom. Explain only from the lecture material in \
         the next message. If the material does not cover something, say so instead of guessing. Keep \
         the reply to a few short sentences. Speak only as the teacher; do not write lines for the \
         students or the user.\n\n{task}"
    )
}

/// The shared user-message body: material, summary and recent conversation. Used by every builder
/// that reads the lecture, so each persona sees the same facts.
pub(crate) fn render_context(material: &[&str], summary: &str, turns: &[Line<'_>]) -> String {
    let mut out = String::from("Lecture material:\n");
    push_material(&mut out, material);
    out.push_str("\n\nSummary so far:\n");
    out.push_str(or_none(summary));
    out.push_str("\n\nRecent conversation:\n");
    push_lines(&mut out, turns);
    out
}

fn push_material(out: &mut String, material: &[&str]) {
    let parts: Vec<&str> = material.iter().copied().filter(|m| !m.is_empty()).collect();
    if parts.is_empty() {
        out.push_str("(no material)");
    } else {
        out.push_str(&parts.join("\n---\n"));
    }
}

fn push_lines(out: &mut String, turns: &[Line<'_>]) {
    if turns.is_empty() {
        out.push_str("(none yet)");
    }
    for (i, t) in turns.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(t.speaker);
        out.push_str(": ");
        out.push_str(t.text);
    }
}

fn or_none(text: &str) -> &str {
    if text.trim().is_empty() {
        "(none yet)"
    } else {
        text
    }
}

fn message(role: PromptRole, content: String) -> PromptMessage {
    PromptMessage { role, content }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn an_empty_lecture_and_no_history_still_gives_placeholders() {
        let msgs = teacher_messages("Dr. Lin", &[], "", &[], TeacherMove::NextSegment);
        assert!(msgs[1].content.contains("(no material)"));
        assert!(msgs[1].content.contains("(none yet)"));
    }
}
