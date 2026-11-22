//! Prompts and replies for generating an exam from one chunk of notes (STORY-0186). The local model is
//! asked for JSON; the reply is read here as plain values, and the rules that accept it are in
//! `studyfrog-rehearsal`. Pure.

use serde::Deserialize;

use crate::rubric_eval::json_span;

/// One question as the model wrote it, before any check.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawQuestion {
    /// `recall`, `case` or `calculation`.
    pub kind: String,
    /// What is asked.
    pub prompt: String,
    /// The reference answer.
    pub sample_answer: String,
    /// The points to look for.
    #[serde(default)]
    pub rubric: Vec<String>,
    /// Points available.
    pub points: u32,
}

#[derive(Deserialize)]
struct Reply {
    questions: Vec<RawQuestion>,
}

/// The instructions for one chunk: the number of each kind to write, the rules, and the JSON shape.
pub fn chunk_system(recall: usize, case: usize, calc: usize) -> String {
    format!(
        "You write questions for a university exam from the course notes you are given. Use only what the notes \
         say: no outside facts. Write {recall} recall questions (definitions and relations), {case} methodological \
         case scenarios (a short situation and a decision to justify), and {calc} calculation exercises with \
         explicit numbers and a worked reference solution. Every question has a reference answer that the notes \
         support, a rubric of the points a good answer makes, and points from 1 to 10. Keep each question to one \
         line. Reply with ONE JSON object on its own: {{\"questions\": [{{\"kind\": \"recall|case|calculation\", \
         \"prompt\": \"...\", \"sample_answer\": \"...\", \"rubric\": [\"...\"], \"points\": 2}}]}}."
    )
}

/// The notes of one chunk, with their topic, as the user's message.
pub fn chunk_user(topic: &str, text: &str) -> String {
    format!("Topic: {topic}\n\nNotes:\n{text}")
}

/// The questions in a reply. A reply with no readable JSON is an error; questions that are malformed
/// are the caller's to reject.
pub fn parse_questions(reply: &str) -> Result<Vec<RawQuestion>, String> {
    let json = json_span(reply).ok_or_else(|| "the reply had no JSON object".to_owned())?;
    let parsed: Reply = serde_json::from_str(json)
        .map_err(|e| format!("the reply is not the expected JSON: {e}"))?;
    Ok(parsed.questions)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_reply_with_questions_is_read() {
        let reply = r#"Here you go: {"questions": [{"kind": "recall", "prompt": "What is reliability?", "sample_answer": "Consistency.", "rubric": ["consistency"], "points": 2}]} done"#;
        let qs = parse_questions(reply).unwrap();
        assert_eq!(qs.len(), 1);
        assert_eq!(qs[0].kind, "recall");
        assert_eq!(qs[0].points, 2);
    }

    #[test]
    fn a_reply_without_json_is_an_error() {
        assert!(parse_questions("no questions today").is_err());
    }

    #[test]
    fn the_instructions_state_the_counts_and_the_shape() {
        let s = chunk_system(2, 1, 1);
        assert!(
            s.contains("Write 2 recall questions")
                && s.contains("1 methodological")
                && s.contains("\"questions\"")
        );
    }
}
