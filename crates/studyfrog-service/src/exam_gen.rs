//! Generating a mock exam from course notes (STORY-0186), in four stages: the notes are cut into
//! topic chunks; the local model writes questions for each chunk (or templates stand in when no model
//! is given or a chunk fails); every question is checked against its topic's notes; the questions are
//! balanced to the 40/30/30 plan and assembled into a mock exam with its reference answers.

use studyfrog_agent::exam_gen::{RawQuestion, chunk_system, chunk_user, parse_questions};
use studyfrog_rehearsal::curriculum::{Chunk, chunk_markdown};
use studyfrog_rehearsal::curriculum_build::{
    AnswerKey, GeneratedQuestion, Kind, Plan, accept, assemble, plan, templates,
};
use studyfrog_rehearsal::mock_exam::MockExam;

/// The model as the generator asks it: the system instructions and the user's message, in; the reply, out.
pub type Ask<'a> = dyn FnMut(&str, &str) -> Result<String, String> + 'a;

/// The result: the exam, its reference answers, and what the stages did with each question.
#[derive(Debug, Clone, PartialEq)]
pub struct Generated {
    /// The exam, ready to run.
    pub exam: MockExam,
    /// The reference answer and scoring of each question, in the exam's order.
    pub keys: Vec<AnswerKey>,
    /// Questions the model wrote that the notes support.
    pub from_model: usize,
    /// Questions made from templates (no model, a failed chunk, or too few from the model).
    pub from_templates: usize,
    /// Why questions were rejected or a chunk failed, one line each.
    pub rejected: Vec<String>,
}

/// Make an exam of about `total` questions from `chunks`. With no `ask`, everything comes from templates.
pub fn generate(
    chunks: &[Chunk],
    total: usize,
    title: &str,
    mut ask: Option<&mut Ask<'_>>,
) -> Generated {
    let want = plan(total);
    let mut accepted: Vec<GeneratedQuestion> = Vec::new();
    let mut rejected = Vec::new();
    if let Some(ask) = ask.as_mut() {
        for (chunk, quota) in chunks.iter().zip(per_chunk(want, chunks.len())) {
            if quota.recall + quota.case + quota.calc == 0 {
                continue;
            }
            let system = chunk_system(quota.recall, quota.case, quota.calc);
            let user = chunk_user(&chunk.topic, &chunk.text);
            match ask(&system, &user).and_then(|reply| parse_questions(&reply)) {
                Ok(raws) => {
                    for raw in raws {
                        judge(raw, chunk, chunks, &mut accepted, &mut rejected);
                    }
                }
                Err(e) => rejected.push(format!("{}: {e}", chunk.topic)),
            }
        }
    }
    let from_model = accepted.len();
    let mut chosen = balance(accepted, want);
    let missing = Plan {
        recall: want.recall.saturating_sub(count(&chosen, Kind::Recall)),
        case: want.case.saturating_sub(count(&chosen, Kind::Case)),
        calc: want.calc.saturating_sub(count(&chosen, Kind::Calculation)),
    };
    let fills = templates(chunks, missing);
    let from_templates = fills.len();
    chosen.extend(fills);
    let (exam, keys) = assemble(title, &chosen);
    Generated {
        exam,
        keys,
        from_model,
        from_templates,
        rejected,
    }
}

/// One raw question, checked: kept when the notes support it, otherwise its reason is noted.
fn judge(
    raw: RawQuestion,
    chunk: &Chunk,
    chunks: &[Chunk],
    accepted: &mut Vec<GeneratedQuestion>,
    rejected: &mut Vec<String>,
) {
    let kind = match raw.kind.trim().to_lowercase().as_str() {
        "recall" => Kind::Recall,
        "case" => Kind::Case,
        "calculation" => Kind::Calculation,
        other => {
            rejected.push(format!("{}: unknown kind {other:?}", chunk.topic));
            return;
        }
    };
    let q = GeneratedQuestion {
        kind,
        prompt: raw.prompt,
        sample_answer: raw.sample_answer,
        rubric: raw.rubric,
        points: u8::try_from(raw.points).unwrap_or(0),
        topic: chunk.topic.clone(),
    };
    match accept(&q, chunks) {
        Ok(()) => accepted.push(q),
        Err(e) => rejected.push(format!("{}: {e}", chunk.topic)),
    }
}

/// The plan spread over the chunks: each chunk gets its share of each kind, the first chunks the extras.
fn per_chunk(want: Plan, n: usize) -> Vec<Plan> {
    let share = |total: usize, i: usize| total / n + usize::from(i < total % n);
    (0..n)
        .map(|i| Plan {
            recall: share(want.recall, i),
            case: share(want.case, i),
            calc: share(want.calc, i),
        })
        .collect()
}

/// Keep at most the planned number of each kind, in the order the model gave them.
fn balance(qs: Vec<GeneratedQuestion>, want: Plan) -> Vec<GeneratedQuestion> {
    let (mut r, mut c, mut k) = (want.recall, want.case, want.calc);
    qs.into_iter()
        .filter(|q| {
            let slot = match q.kind {
                Kind::Recall => &mut r,
                Kind::Case => &mut c,
                Kind::Calculation => &mut k,
            };
            if *slot > 0 {
                *slot -= 1;
                true
            } else {
                false
            }
        })
        .collect()
}

fn count(qs: &[GeneratedQuestion], kind: Kind) -> usize {
    qs.iter().filter(|q| q.kind == kind).count()
}

/// The notes of a Markdown file as chunks (a convenience for callers that have the text).
pub fn chunks_of_markdown(md: &str) -> Vec<Chunk> {
    chunk_markdown(md)
}

/// The text of a lecture PDF as chunks. A PDF has no Markdown headings, so its text is one topic
/// ([`studyfrog_rehearsal::curriculum::GENERAL`]) unless its text carries headings of its own.
pub fn chunks_of_pdf(bytes: &[u8]) -> Result<Vec<Chunk>, String> {
    let pages = studyfrog_pdf::text::pdf_all_text(&std::sync::Arc::new(bytes.to_vec()))
        .map_err(|e| studyfrog_pdf::error::describe_pdf_error(&e))?;
    let text = pages
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    Ok(chunk_markdown(&text))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    const NOTES: &str = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n";

    fn json(questions: &str) -> String {
        format!("{{\"questions\": [{questions}]}}")
    }

    #[test]
    fn offline_the_exam_comes_from_templates_and_is_valid() {
        let g = generate(&chunks_of_markdown(NOTES), 6, "Klausur", None);
        assert_eq!(g.from_model, 0);
        assert!(g.from_templates > 0);
        assert!(studyfrog_rehearsal::mock_exam::validate_mock_exam(&g.exam).is_ok());
        assert_eq!(
            g.keys.len(),
            g.exam
                .sections
                .iter()
                .map(|s| s.questions.len())
                .sum::<usize>()
        );
    }

    #[test]
    fn the_model_questions_that_the_notes_support_are_kept_and_the_rest_rejected() {
        let good = r#"{"kind": "recall", "prompt": "What does the reliability coefficient compare?", "sample_answer": "True score variance with observed score variance.", "rubric": ["names both variances"], "points": 2}"#;
        let invented = r#"{"kind": "recall", "prompt": "Describe the Rorschach inkblot method in detail.", "sample_answer": "Projective inkblots reveal unconscious conflicts through free association.", "rubric": [], "points": 2}"#;
        let reply = json(&format!("{good}, {invented}"));
        let mut ask = |_: &str, _: &str| Ok(reply.clone());
        let g = generate(&chunks_of_markdown(NOTES), 3, "Klausur", Some(&mut ask));
        assert_eq!(g.from_model, 1, "{:?}", g.rejected);
        assert!(g.rejected.iter().any(|r| r.contains("hallucination")));
    }

    #[test]
    fn a_failed_chunk_is_noted_and_templates_stand_in() {
        let mut ask = |_: &str, _: &str| Err("the model is not running".to_owned());
        let g = generate(&chunks_of_markdown(NOTES), 3, "Klausur", Some(&mut ask));
        assert_eq!(g.from_model, 0);
        assert!(g.rejected.iter().any(|r| r.contains("not running")));
        assert!(studyfrog_rehearsal::mock_exam::validate_mock_exam(&g.exam).is_ok());
    }
}
