//! Deterministic objective grading for single-select, multiple-select, and fill-in-the-gaps items (STORY-0207).

use crate::curriculum_build::AnswerKey;
use crate::exam_answer::{ExamAnswer, choice_tag};
use crate::exam_format::ExamItemFormat;

/// Result of deterministic objective grading for one question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveGrade {
    /// Points awarded (`0..=possible`).
    pub earned: u32,
    /// Maximum points for this question.
    pub possible: u32,
    /// Per-gap or per-choice verification items (`(description, ok)`).
    pub checks: Vec<(String, bool)>,
}

/// Normalize a gap fill for comparison: trim, collapse whitespace, lowercase, strip trailing punctuation.
pub fn normalize_gap(raw: &str) -> String {
    let collapsed = raw
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    collapsed
        .trim_end_matches(['.', ',', ';', ':', '!', '?'])
        .trim()
        .to_owned()
}

/// Deterministic all-or-nothing grading for single-select questions.
pub fn grade_single_select(
    selected: Option<usize>,
    correct_indices: &[usize],
    options: &[String],
    possible: u32,
) -> ObjectiveGrade {
    let expected = correct_indices.first().copied();
    let expected_label = expected
        .and_then(|i| options.get(i).map(|o| format!("{}. {o}", choice_tag(i))))
        .unwrap_or_else(|| "Valid option".to_owned());
    let ok = selected.is_some() && selected == expected;
    let earned = if ok { possible } else { 0 };
    ObjectiveGrade {
        earned,
        possible,
        checks: vec![(format!("Expected {expected_label}"), ok)],
    }
}

/// Deterministic net partial-credit grading for multiple-select questions:
/// `max(0, hits - wrong) / correct_count`, rounded to nearest point.
pub fn grade_multiple_select(
    selected: &[bool],
    correct_indices: &[usize],
    options: &[String],
    possible: u32,
) -> ObjectiveGrade {
    if correct_indices.is_empty() {
        return ObjectiveGrade {
            earned: 0,
            possible,
            checks: Vec::new(),
        };
    }
    let mut hits: i32 = 0;
    let mut wrong: i32 = 0;
    let mut checks = Vec::new();
    for (idx, opt) in options.iter().enumerate() {
        let picked = selected.get(idx).copied().unwrap_or(false);
        let is_correct = correct_indices.contains(&idx);
        let tag = choice_tag(idx);
        if is_correct && picked {
            hits += 1;
            checks.push((format!("Selected {tag}. {opt}"), true));
        } else if is_correct && !picked {
            checks.push((format!("Missed {tag}. {opt}"), false));
        } else if !is_correct && picked {
            wrong += 1;
            checks.push((format!("Wrongly selected {tag}. {opt}"), false));
        }
    }
    let net = (hits - wrong).max(0) as f32;
    let share = (net / correct_indices.len() as f32).clamp(0.0, 1.0);
    let earned = (possible as f32 * share).round() as u32;
    ObjectiveGrade {
        earned: earned.min(possible),
        possible,
        checks,
    }
}

/// Deterministic per-gap grading using [`normalize_gap`].
pub fn grade_fill_in_gaps(
    given: &[String],
    expected_gaps: &[String],
    possible: u32,
) -> ObjectiveGrade {
    if expected_gaps.is_empty() {
        return ObjectiveGrade {
            earned: 0,
            possible,
            checks: Vec::new(),
        };
    }
    let mut matched = 0usize;
    let mut checks = Vec::with_capacity(expected_gaps.len());
    for (i, exp) in expected_gaps.iter().enumerate() {
        let user_text = given.get(i).map(String::as_str).unwrap_or("");
        let ok = !exp.trim().is_empty() && normalize_gap(user_text) == normalize_gap(exp);
        if ok {
            matched += 1;
        }
        checks.push((format!("Gap [{}]: {}", i + 1, exp.trim()), ok));
    }
    let share = matched as f32 / expected_gaps.len() as f32;
    let earned = (possible as f32 * share).round() as u32;
    ObjectiveGrade {
        earned: earned.min(possible),
        possible,
        checks,
    }
}

/// Grade an objective format (`FillInGaps`, `SingleSelect`, `MultipleSelect`), returning `None` for open/calc/case.
pub fn grade_objective(
    format: ExamItemFormat,
    answer: &ExamAnswer,
    key: &AnswerKey,
    options: &[String],
    possible: u32,
) -> Option<ObjectiveGrade> {
    match format {
        ExamItemFormat::SingleSelect => {
            let selected = match answer {
                ExamAnswer::Single(s) => *s,
                _ => None,
            };
            Some(grade_single_select(
                selected,
                &key.correct_indices,
                options,
                possible,
            ))
        }
        ExamItemFormat::MultipleSelect => {
            let empty = Vec::new();
            let flags = match answer {
                ExamAnswer::Multiple(v) => v.as_slice(),
                _ => empty.as_slice(),
            };
            Some(grade_multiple_select(
                flags,
                &key.correct_indices,
                options,
                possible,
            ))
        }
        ExamItemFormat::FillInGaps => {
            let empty = Vec::new();
            let gaps = match answer {
                ExamAnswer::Gaps(v) => v.as_slice(),
                _ => empty.as_slice(),
            };
            Some(grade_fill_in_gaps(gaps, &key.gap_answers, possible))
        }
        ExamItemFormat::OpenQuestion
        | ExamItemFormat::Calculation
        | ExamItemFormat::CaseScenario => None,
    }
}
