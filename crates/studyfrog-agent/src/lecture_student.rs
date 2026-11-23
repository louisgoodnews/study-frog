//! Prompt builder for a simulated student in a lecture (STORY-0209, TASK-0289). A persona is plain
//! data: a name and a stance. The student asks one grounded question and never explains or answers.
//! Pure.

use crate::lecture_prompt::{Line, PromptMessage, PromptRole, render_context};

/// A simulated student: a display name and the stance that shapes the questions they ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Persona<'a> {
    /// Display name.
    pub name: &'a str,
    /// How the student thinks, in one clause, e.g. "a curious beginner who asks what a term means".
    pub stance: &'a str,
}

/// Three distinct built-in personas: a curious beginner, a detail-oriented skeptic and a practical
/// applier. Callers may pass their own [`Persona`] instead.
pub const PRESET_PERSONAS: [Persona<'static>; 3] = [
    Persona {
        name: "Ada",
        stance: "a curious beginner who asks what a term means and why it matters",
    },
    Persona {
        name: "Ben",
        stance: "a detail-oriented skeptic who asks for the exact condition, edge case or evidence behind a claim",
    },
    Persona {
        name: "Mia",
        stance: "a practical applier who asks how the idea works on a concrete example",
    },
];

/// The system and user messages for one simulated student's question. `asked_before` lists the
/// questions this student already asked, so the new one does not repeat them.
pub fn student_messages(
    persona: &Persona<'_>,
    material: &[&str],
    summary: &str,
    turns: &[Line<'_>],
    asked_before: &[&str],
) -> Vec<PromptMessage> {
    let system = format!(
        "You are {}, a student in a study classroom. You are {}. Ask exactly one question about the \
         lecture material, in character, in one sentence. Do not answer questions and do not explain \
         the material. Do not repeat a question from the list in the next message.",
        persona.name, persona.stance
    );
    let mut user = render_context(material, summary, turns);
    user.push_str("\n\nQuestions you already asked:\n");
    if asked_before.is_empty() {
        user.push_str("(none yet)");
    }
    for q in asked_before {
        user.push_str("- ");
        user.push_str(q);
        user.push('\n');
    }
    vec![
        PromptMessage {
            role: PromptRole::System,
            content: system,
        },
        PromptMessage {
            role: PromptRole::User,
            content: user,
        },
    ]
}
