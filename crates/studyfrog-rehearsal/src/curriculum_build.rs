//! Building a mock exam from curriculum chunks (STORY-0186): the 40/30/30 plan, the filter that rejects
//! items the notes do not support, the offline templates, and the assembly into a [`MockExam`]. Pure.

use crate::curriculum::{Chunk, significant_words};
use crate::mock_exam::{ExamQuestion, ExamSection, MockExam, QuestionPoints};

pub use crate::curriculum_templates::templates;

/// Kinds of question, in the exam's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Conceptual recall: definitions and relations.
    Recall,
    /// A methodological case scenario.
    Case,
    /// A calculation with a reference solution.
    Calculation,
}

/// One question as generated (by the model or a template), before it is filtered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedQuestion {
    /// The kind.
    pub kind: Kind,
    /// What is asked (one line).
    pub prompt: String,
    /// The reference answer.
    pub sample_answer: String,
    /// The points the answer must make, one per line.
    pub rubric: Vec<String>,
    /// Points available.
    pub points: u8,
    /// The topic (a chunk's heading).
    pub topic: String,
}

/// How many questions of each kind an exam has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// Recall questions.
    pub recall: usize,
    /// Case scenarios.
    pub case: usize,
    /// Calculations.
    pub calc: usize,
}

/// The reference answer and scoring of one question, kept beside the exam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerKey {
    /// The question it belongs to.
    pub prompt: String,
    /// The reference answer.
    pub sample_answer: String,
    /// The points to look for.
    pub rubric: Vec<String>,
}

/// The mix the exam aims at: 40 % recall, 30 % cases, 30 % calculations (at least one of each when there
/// are three or more questions).
pub fn plan(total: usize) -> Plan {
    let recall = (total as f64 * 0.4).round() as usize;
    let case = (total as f64 * 0.3).round() as usize;
    let calc = total.saturating_sub(recall + case);
    let mut p = Plan { recall, case, calc };
    if total >= 3 {
        for slot in [&mut p.recall, &mut p.case, &mut p.calc] {
            if *slot == 0 {
                *slot = 1;
            }
        }
        let over = (p.recall + p.case + p.calc).saturating_sub(total);
        p.recall = p.recall.saturating_sub(over);
    }
    p
}

/// Why a generated question is rejected, or `Ok` when the notes support it.
pub fn accept(q: &GeneratedQuestion, chunks: &[Chunk]) -> Result<(), String> {
    let prompt = q.prompt.trim();
    if !(10..=400).contains(&prompt.chars().count()) || prompt.contains('\n') {
        return Err("the question must be one line of 10 to 400 characters".to_owned());
    }
    if !(1..=10).contains(&q.points) {
        return Err("points must be between 1 and 10".to_owned());
    }
    if q.sample_answer.trim().is_empty() {
        return Err("a question needs a reference answer".to_owned());
    }
    let Some(chunk) = chunks
        .iter()
        .find(|c| c.topic.eq_ignore_ascii_case(q.topic.trim()))
    else {
        return Err(format!(
            "the topic {:?} is not in the notes (out of syllabus)",
            q.topic
        ));
    };
    let source = significant_words(&chunk.text);
    let answer = significant_words(&q.sample_answer);
    if answer.is_empty() {
        return Err("the reference answer names nothing from the notes".to_owned());
    }
    let grounded = answer.iter().filter(|w| source.contains(w)).count();
    if grounded * 10 < answer.len() * 4 {
        return Err(
            "the reference answer is not supported by the notes (possible hallucination)"
                .to_owned(),
        );
    }
    Ok(())
}

/// Put the questions into the exam's sections, in kind order, and keep their reference answers beside it.
/// The time limit follows the points: three minutes a point, within the allowed range.
pub fn assemble(title: &str, questions: &[GeneratedQuestion]) -> (MockExam, Vec<AnswerKey>) {
    let names = [
        (Kind::Recall, "Part A: conceptual recall"),
        (Kind::Case, "Part B: case scenarios"),
        (Kind::Calculation, "Part C: calculations"),
    ];
    let mut sections = Vec::new();
    let mut keys = Vec::new();
    for (kind, name) in names {
        let qs: Vec<&GeneratedQuestion> = questions.iter().filter(|q| q.kind == kind).collect();
        if qs.is_empty() {
            continue;
        }
        sections.push(ExamSection {
            name: name.to_owned(),
            questions: qs
                .iter()
                .map(|q| ExamQuestion {
                    prompt: q.prompt.trim().to_owned(),
                    topic: q.topic.clone(),
                    points: QuestionPoints(q.points),
                })
                .collect(),
        });
        keys.extend(qs.iter().map(|q| AnswerKey {
            prompt: q.prompt.trim().to_owned(),
            sample_answer: q.sample_answer.clone(),
            rubric: q.rubric.clone(),
        }));
    }
    let points: u32 = questions.iter().map(|q| u32::from(q.points)).sum();
    let time = (points * 3).clamp(
        crate::mock_exam::MIN_TIME_LIMIT_MIN,
        crate::mock_exam::MAX_TIME_LIMIT_MIN,
    );
    (
        MockExam {
            title: title.to_owned(),
            time_limit_min: time,
            sections,
        },
        keys,
    )
}
