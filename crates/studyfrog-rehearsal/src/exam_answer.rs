//! Structured mock exam student answers and formatting helpers (STORY-0207). Pure.

pub use crate::exam_answer_grade::{
    ObjectiveGrade, grade_fill_in_gaps, grade_multiple_select, grade_objective,
    grade_single_select, normalize_gap,
};

use crate::curriculum_build::AnswerKey;
use crate::exam_format::ExamItemFormat;
use crate::mock_exam::ExamQuestion;

/// Explicit scoring rule displayed in graded review when multiple-select items appear.
pub const MULTI_SELECT_RULE: &str = "Multiple-select grading rule: each correct choice earns +1/(correct choices), each wrong choice deducts -1/(correct choices), floored at 0 points per question.";

/// Student answer payload matching the question's [`ExamItemFormat`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExamAnswer {
    /// Free text for open questions, calculations, or case scenarios.
    Text(String),
    /// Ordered answers for fill-in-the-gaps blanks.
    Gaps(Vec<String>),
    /// Selected option index for single-select questions.
    Single(Option<usize>),
    /// Checkbox toggles aligned with `ExamQuestion::options` for multiple-select questions.
    Multiple(Vec<bool>),
}

impl Default for ExamAnswer {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// Blank answer initialized to match `q`'s format, choice count, or gap count.
pub fn blank_answer(q: &ExamQuestion) -> ExamAnswer {
    match q.format {
        ExamItemFormat::OpenQuestion
        | ExamItemFormat::Calculation
        | ExamItemFormat::CaseScenario => ExamAnswer::Text(String::new()),
        ExamItemFormat::FillInGaps => ExamAnswer::Gaps(vec![String::new(); q.gap_hints.len()]),
        ExamItemFormat::SingleSelect => ExamAnswer::Single(None),
        ExamItemFormat::MultipleSelect => ExamAnswer::Multiple(vec![false; q.options.len()]),
    }
}

/// Whether `format` is graded deterministically rather than via SLM or word overlap.
pub fn is_objective_format(format: ExamItemFormat) -> bool {
    matches!(
        format,
        ExamItemFormat::FillInGaps | ExamItemFormat::SingleSelect | ExamItemFormat::MultipleSelect
    )
}

/// Choice letter tag (`A`, `B`, `C`, ...) for option `idx`.
pub fn choice_tag(idx: usize) -> String {
    if idx < 26 {
        let ch = (b'A' + idx as u8) as char;
        ch.to_string()
    } else {
        format!("{}", idx + 1)
    }
}

/// Render a student's [`ExamAnswer`] into a readable string for graded review.
pub fn format_answer_text(answer: &ExamAnswer, options: &[String]) -> String {
    match answer {
        ExamAnswer::Text(s) => {
            let t = s.trim();
            if t.is_empty() {
                "(none)".to_owned()
            } else {
                t.to_owned()
            }
        }
        ExamAnswer::Gaps(gaps) => {
            if gaps.iter().all(|g| g.trim().is_empty()) {
                return "(none)".to_owned();
            }
            gaps.iter()
                .enumerate()
                .map(|(i, g)| {
                    let t = g.trim();
                    let shown = if t.is_empty() { "—" } else { t };
                    format!("[{}] {shown}", i + 1)
                })
                .collect::<Vec<_>>()
                .join("; ")
        }
        ExamAnswer::Single(Some(idx)) => options.get(*idx).map_or_else(
            || "(none)".to_owned(),
            |o| format!("{}. {o}", choice_tag(*idx)),
        ),
        ExamAnswer::Single(None) => "(none)".to_owned(),
        ExamAnswer::Multiple(flags) => {
            let picked: Vec<String> = flags
                .iter()
                .enumerate()
                .filter_map(|(i, &on)| {
                    on.then(|| options.get(i).map(|o| format!("{}. {o}", choice_tag(i))))
                        .flatten()
                })
                .collect();
            if picked.is_empty() {
                "(none)".to_owned()
            } else {
                picked.join("; ")
            }
        }
    }
}

/// Render the expected reference answer for graded review.
pub fn format_expected_text(key: &AnswerKey, options: &[String]) -> String {
    match key.format {
        ExamItemFormat::SingleSelect | ExamItemFormat::MultipleSelect => {
            let parts: Vec<String> = key
                .correct_indices
                .iter()
                .filter_map(|&i| options.get(i).map(|o| format!("{}. {o}", choice_tag(i))))
                .collect();
            if parts.is_empty() {
                key.sample_answer.clone()
            } else {
                parts.join("; ")
            }
        }
        ExamItemFormat::FillInGaps => {
            if key.gap_answers.is_empty() {
                key.sample_answer.clone()
            } else {
                key.gap_answers
                    .iter()
                    .enumerate()
                    .map(|(i, a)| format!("[{}] {}", i + 1, a.trim()))
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        }
        ExamItemFormat::OpenQuestion
        | ExamItemFormat::Calculation
        | ExamItemFormat::CaseScenario => key.sample_answer.clone(),
    }
}
