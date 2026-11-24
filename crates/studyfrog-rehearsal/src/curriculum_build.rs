//! Building a mock exam from curriculum chunks (STORY-0186, STORY-0207): format plan, grounding filter,
//! offline templates, and assembly into a [`MockExam`]. Pure.

use std::collections::HashSet;

use crate::curriculum::{Chunk, significant_words};
use crate::curriculum_gaps::valid_gap_markers;
pub use crate::curriculum_templates::{material, templates};
pub use crate::exam_format::{ExamItemFormat, FormatPlan, legacy_plan as plan};
use crate::exam_format::{plan_count, section_heading, section_order};
use crate::mock_exam::{ExamQuestion, ExamSection, MockExam, QuestionPoints};

/// One question as generated (by the model or a template), before it is filtered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedQuestion {
    /// Item format.
    pub format: ExamItemFormat,
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
    /// Answer options for single/multiple select.
    pub options: Vec<String>,
    /// 0-based indices of correct options for single/multiple select.
    pub correct_indices: Vec<usize>,
    /// Optional per-gap hints for fill-in-the-gaps.
    pub gap_hints: Vec<Option<String>>,
    /// Expected answers for fill-in-the-gaps.
    pub gap_answers: Vec<String>,
}

/// The reference answer and scoring of one question, kept beside the exam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerKey {
    /// Item format.
    pub format: ExamItemFormat,
    /// The question it belongs to.
    pub prompt: String,
    /// The reference answer.
    pub sample_answer: String,
    /// The points to look for.
    pub rubric: Vec<String>,
    /// 0-based indices of correct options for single/multiple select.
    pub correct_indices: Vec<usize>,
    /// Expected answers for fill-in-the-gaps.
    pub gap_answers: Vec<String>,
}

/// A part of the exam with fewer questions than the plan asks for (BUG-0055).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatShort {
    /// The format part.
    pub format: ExamItemFormat,
    /// Questions the plan asks for.
    pub wanted: usize,
    /// Questions the exam has.
    pub made: usize,
}

/// The parts short of the plan, in `Part A`..`Part F` order. Empty when every part is filled.
pub fn shortfalls(want: FormatPlan, made: FormatPlan) -> Vec<FormatShort> {
    section_order()
        .into_iter()
        .filter_map(|format| {
            let wanted = plan_count(&want, format);
            let have = plan_count(&made, format);
            (have < wanted).then_some(FormatShort {
                format,
                wanted,
                made: have,
            })
        })
        .collect()
}

/// The line the exam shows for a short part: how many it has, and what the notes lack.
pub fn short_note(s: &FormatShort) -> String {
    let (part, lacks) = match s.format {
        ExamItemFormat::OpenQuestion => (
            "Part A (conceptual recall)",
            "definitions (a line \"Term: meaning\" or \"X ist ...\")",
        ),
        ExamItemFormat::CaseScenario => ("Part B (case scenarios)", "paragraphs of text"),
        ExamItemFormat::Calculation => (
            "Part C (calculations)",
            "formulas (a line with \"=\" and a digit)",
        ),
        ExamItemFormat::FillInGaps => (
            "Part D (fill in the gaps)",
            "definitions or sentences with key terms to mask",
        ),
        ExamItemFormat::SingleSelect => (
            "Part E (single select)",
            "distinct definitions or statements (at least 2)",
        ),
        ExamItemFormat::MultipleSelect => (
            "Part F (multiple select)",
            "distinct definitions or statements (at least 2)",
        ),
    };
    format!(
        "{part} has {} of the {} questions the plan asks for: the notes have too few {lacks}. Add them to the notes to fill it.",
        s.made, s.wanted
    )
}

/// Why a generated question is rejected, or `Ok` when the notes and format rules support it.
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
    accept_format(q, prompt)?;
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

fn accept_format(q: &GeneratedQuestion, prompt: &str) -> Result<(), String> {
    match q.format {
        ExamItemFormat::SingleSelect | ExamItemFormat::MultipleSelect => {
            if q.options.len() < 2 {
                return Err("select questions need at least two options".to_owned());
            }
            let mut seen = HashSet::new();
            for opt in &q.options {
                let norm = opt.trim().to_lowercase();
                if norm.is_empty() || !seen.insert(norm) {
                    return Err("select options must be non-empty and distinct".to_owned());
                }
            }
            let mut idx_seen = HashSet::new();
            if q.correct_indices.is_empty()
                || (q.format == ExamItemFormat::SingleSelect && q.correct_indices.len() != 1)
                || q.correct_indices
                    .iter()
                    .any(|&i| i >= q.options.len() || !idx_seen.insert(i))
            {
                return Err("invalid correct option indices".to_owned());
            }
        }
        ExamItemFormat::FillInGaps => {
            if q.gap_answers.is_empty() || q.gap_answers.len() != q.gap_hints.len() {
                return Err("fill-in-the-gaps requires matching gap answers and hints".to_owned());
            }
            if q.gap_answers.iter().any(|a| a.trim().is_empty()) {
                return Err("gap answers must not be empty".to_owned());
            }
            if prompt.contains("{{c") || !valid_gap_markers(prompt, q.gap_answers.len()) {
                return Err(
                    "fill-in-the-gaps prompt blanks must match gap answers [1]..[n]".to_owned(),
                );
            }
        }
        _ => {}
    }
    Ok(())
}

/// Put the questions into the exam's sections (`Part A`..`Part F`) and keep their reference answers beside it.
pub fn assemble(title: &str, questions: &[GeneratedQuestion]) -> (MockExam, Vec<AnswerKey>) {
    let mut sections = Vec::new();
    let mut keys = Vec::new();
    for format in section_order() {
        let qs: Vec<&GeneratedQuestion> = questions.iter().filter(|q| q.format == format).collect();
        if qs.is_empty() {
            continue;
        }
        sections.push(ExamSection {
            name: section_heading(format).to_owned(),
            questions: qs
                .iter()
                .map(|q| ExamQuestion {
                    format: q.format,
                    prompt: q.prompt.trim().to_owned(),
                    topic: q.topic.clone(),
                    points: QuestionPoints(q.points),
                    options: q.options.clone(),
                    gap_hints: q.gap_hints.clone(),
                })
                .collect(),
        });
        keys.extend(qs.iter().map(|q| AnswerKey {
            format: q.format,
            prompt: q.prompt.trim().to_owned(),
            sample_answer: q.sample_answer.clone(),
            rubric: q.rubric.clone(),
            correct_indices: q.correct_indices.clone(),
            gap_answers: q.gap_answers.clone(),
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
