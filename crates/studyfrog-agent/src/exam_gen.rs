//! Prompts and replies for generating an exam from one chunk of notes (STORY-0186, STORY-0207).
//! The local model is asked for JSON; the reply is read here as plain values, and the rules that accept
//! it are in `studyfrog-rehearsal`. Pure.

use serde::Deserialize;

use crate::rubric_eval::json_span;

/// Requested question counts per item format for one chunk prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatCounts {
    /// Open-ended conceptual recall questions.
    pub open_question: usize,
    /// Fill-in-the-gaps / cloze questions.
    pub fill_in_gaps: usize,
    /// Single-select questions.
    pub single_select: usize,
    /// Multiple-select questions.
    pub multiple_select: usize,
    /// Calculation problems.
    pub calculation: usize,
    /// Case scenario vignettes.
    pub case_scenario: usize,
}

/// One question as the model wrote it, before any check.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawQuestion {
    /// Wire format name or legacy kind (`open_question`, `fill_in_gaps`, `single_select`, `multiple_select`, `calculation`, `case_scenario`).
    #[serde(alias = "format")]
    pub kind: String,
    /// What is asked.
    pub prompt: String,
    /// The reference answer.
    #[serde(default)]
    pub sample_answer: String,
    /// The points to look for.
    #[serde(default)]
    pub rubric: Vec<String>,
    /// Points available.
    pub points: u32,
    /// Choice texts for single/multiple select.
    #[serde(default)]
    pub options: Vec<String>,
    /// 0-based indices of correct options.
    #[serde(default)]
    pub correct_indices: Vec<usize>,
    /// Optional single correct index fallback from models.
    #[serde(default)]
    pub correct_index: Option<usize>,
    /// Ordered gap answers for fill-in-the-gaps.
    #[serde(default)]
    pub gaps: Vec<String>,
}

#[derive(Deserialize)]
struct Reply {
    questions: Vec<RawQuestion>,
}

/// Format-aware instructions for one chunk: emits rules and JSON shape for each format with `count > 0`.
pub fn chunk_system_for_formats(c: FormatCounts) -> String {
    let mut specs = Vec::new();
    let mut kinds = Vec::new();
    if c.open_question > 0 {
        specs.push(format!(
            "{} open_question items (definitions and relations)",
            c.open_question
        ));
        kinds.push("open_question");
    }
    if c.fill_in_gaps > 0 {
        specs.push(format!(
            "{} fill_in_gaps items (prompt with [1], [2] or {{{{c1::answer}}}} blanks and a \"gaps\" array of expected answers)",
            c.fill_in_gaps
        ));
        kinds.push("fill_in_gaps");
    }
    if c.single_select > 0 {
        specs.push(format!(
            "{} single_select items (\"options\" array of 3-4 distinct choices and \"correct_indices\" with 1 index)",
            c.single_select
        ));
        kinds.push("single_select");
    }
    if c.multiple_select > 0 {
        specs.push(format!(
            "{} multiple_select items (\"options\" array of 3-4 distinct choices and \"correct_indices\" with 1+ indices)",
            c.multiple_select
        ));
        kinds.push("multiple_select");
    }
    if c.calculation > 0 {
        specs.push(format!(
            "{} calculation items (explicit numbers and worked reference solution)",
            c.calculation
        ));
        kinds.push("calculation");
    }
    if c.case_scenario > 0 {
        specs.push(format!(
            "{} case_scenario items (short situation and decision to justify)",
            c.case_scenario
        ));
        kinds.push("case_scenario");
    }
    let ask_list = if specs.is_empty() {
        "0 questions".to_owned()
    } else {
        specs.join(", ")
    };
    let kind_union = if kinds.is_empty() {
        "open_question".to_owned()
    } else {
        kinds.join("|")
    };
    format!(
        "You write questions for a university exam from the course notes you are given. Use only what the notes \
         say: no outside facts. Write {ask_list}. Every question has a grounded reference answer, a rubric of key \
         points, and points from 1 to 10. Keep each prompt to one line. Reply with ONE JSON object on its own: \
         {{\"questions\": [{{\"kind\": \"{kind_union}\", \"prompt\": \"...\", \"sample_answer\": \"...\", \
         \"rubric\": [\"...\"], \"points\": 2, \"options\": [], \"correct_indices\": [], \"gaps\": []}}]}}."
    )
}

/// Legacy three-kind wrapper kept for compatibility.
pub fn chunk_system(recall: usize, case: usize, calc: usize) -> String {
    chunk_system_for_formats(FormatCounts {
        open_question: recall,
        case_scenario: case,
        calculation: calc,
        ..FormatCounts::default()
    })
}

/// The notes of one chunk, with their topic, as the user's message.
pub fn chunk_user(topic: &str, text: &str) -> String {
    format!("Topic: {topic}\n\nNotes:\n{text}")
}

/// The questions in a reply. A reply with no readable JSON is an error; malformed items are filtered later.
pub fn parse_questions(reply: &str) -> Result<Vec<RawQuestion>, String> {
    let json = json_span(reply).ok_or_else(|| "the reply had no JSON object".to_owned())?;
    let parsed: Reply = serde_json::from_str(json)
        .map_err(|e| format!("the reply is not the expected JSON: {e}"))?;
    Ok(parsed.questions)
}
