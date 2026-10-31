//! Examining the user: the model asks, the user answers, the model grades. The verdicts come back as
//! one JSON object at the end, and the user decides whether they are recorded as reviews.

use serde_json::Value;

use crate::wiki::json_in;

/// One item the exam may ask about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamItem {
    /// The item's id as text.
    pub uid: String,
    /// The question (a card's front, a question's prompt).
    pub prompt: String,
    /// The reference answer.
    pub reference: String,
}

/// How an answer was judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Right.
    Correct,
    /// Partly right.
    Partial,
    /// Wrong.
    Incorrect,
}

/// The verdict for one item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamResult {
    /// The item's id as text.
    pub uid: String,
    /// The verdict.
    pub verdict: Verdict,
}

/// The instructions of an exam over `items`.
pub fn exam_system(items: &[ExamItem]) -> String {
    let mut out = String::from(
        "You are a fair, friendly examiner. Ask the questions below one at a time, in any order, and wait for the \
         student's answer before moving on. After each answer say briefly whether it was right, partly right or \
         wrong and give the reference answer when it was not fully right. Do not reveal an answer before the \
         student has tried. When all questions have been asked and answered, finish with a short summary and then \
         ONE JSON object on its own, like {\"results\": [{\"id\": \"<id>\", \"verdict\": \"correct\"}]}, with the \
         verdict correct, partial or incorrect for every question you asked. Do not write that JSON before the end.\n\nQuestions:\n",
    );
    for i in items {
        out.push_str(&format!(
            "\n- id={}\n  question: {}\n  reference answer: {}\n",
            i.uid, i.prompt, i.reference
        ));
    }
    out
}

/// The student's first turn, which starts the exam.
pub fn exam_opening() -> String {
    "I am ready. Please start the exam.".to_owned()
}

fn verdict_of(word: &str) -> Option<Verdict> {
    match word.trim().to_lowercase().as_str() {
        "correct" => Some(Verdict::Correct),
        "partial" | "partly" => Some(Verdict::Partial),
        "incorrect" | "wrong" => Some(Verdict::Incorrect),
        _ => None,
    }
}

/// The verdicts in a reply, for the items the exam was about; `None` while the exam is not finished.
pub fn parse_results(reply: &str, uids: &[String]) -> Option<Vec<ExamResult>> {
    let list = json_in(reply)?.get("results")?.as_array().cloned()?;
    let mut out: Vec<ExamResult> = Vec::new();
    for r in &list {
        let id = r.get("id").and_then(Value::as_str)?.to_owned();
        let verdict = r
            .get("verdict")
            .and_then(Value::as_str)
            .and_then(verdict_of)?;
        if uids.contains(&id) && !out.iter().any(|x| x.uid == id) {
            out.push(ExamResult { uid: id, verdict });
        }
    }
    (!out.is_empty()).then_some(out)
}

/// The reply without the JSON object at its end, for showing.
pub fn without_results(reply: &str) -> String {
    match (reply.find("{\"results\""), reply.rfind('}')) {
        (Some(start), Some(end)) if start < end => {
            format!("{}{}", &reply[..start], &reply[end + 1..])
                .trim()
                .to_owned()
        }
        _ => reply.trim().to_owned(),
    }
}
