//! Grading a mock exam (STORY-0187, STORY-0207): objective formats grade deterministically while open
//! questions, calculations, and case scenarios use the local model or keyword fallback. Pure.

use studyfrog_agent::exam_grade::{Ask, grade_answer, is_counted, share_met};
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::exam_answer::{
    ExamAnswer, blank_answer, format_answer_text, format_expected_text, grade_objective,
};
use studyfrog_rehearsal::exam_format::ExamItemFormat;
use studyfrog_rehearsal::grade::{ExamResult, exam_result};
use studyfrog_rehearsal::mock_exam::{MockExam, all_questions};

use super::MockExamUi;

/// Default pass mark percentage ("Ausreichend": 50 %).
pub const DEFAULT_PASS_MARK: f32 = 50.0;

/// How one question was graded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradingEngine {
    /// Deterministic rule-based grading (`FillInGaps`, `SingleSelect`, `MultipleSelect`).
    Objective,
    /// Local SLM rubric grading.
    Model,
    /// Keyword word-overlap fallback.
    WordOverlap,
}

/// The pass mark of an exam in percent: the student's setting, or the default.
pub fn pass_mark_of(ui: &MockExamUi) -> f32 {
    ui.pass_mark.unwrap_or(DEFAULT_PASS_MARK)
}

/// The pass mark as the student may set it: whole percent from 1 to 100.
pub fn clamp_pass_mark(pct: f32) -> f32 {
    if pct.is_finite() {
        pct.round().clamp(1.0, 100.0)
    } else {
        DEFAULT_PASS_MARK
    }
}

/// One question after grading.
#[derive(Debug, Clone, PartialEq)]
pub struct Graded {
    /// Item format of the question.
    pub format: ExamItemFormat,
    /// The question's topic.
    pub topic: String,
    /// The question.
    pub prompt: String,
    /// Formatted student answer.
    pub answer: String,
    /// Formatted reference / expected answer.
    pub sample_answer: String,
    /// Verification checks or rubric points (`Some(true)` met, `Some(false)` missed, `None` unchecked).
    pub points_made: Vec<(String, Option<bool>)>,
    /// Points available.
    pub possible: u32,
    /// Points earned.
    pub earned: u32,
    /// False when no rubric point could be checked: left out of the exam's total.
    pub counted: bool,
    /// True when the local model judged the answer.
    pub from_model: bool,
    /// Engine used to grade this question.
    pub engine: GradingEngine,
    /// What the student should know about this grade, if anything.
    pub caveat: Option<String>,
}

/// Grade every question of `exam` synchronously (objective rules + word-overlap fallback).
pub fn grade(exam: &MockExam, keys: &[AnswerKey], answers: &[ExamAnswer]) -> Vec<Graded> {
    grade_with(exam, keys, answers, None)
}

/// Grade every question of `exam` against its key, using `ask` for non-objective questions when provided.
pub fn grade_with(
    exam: &MockExam,
    keys: &[AnswerKey],
    answers: &[ExamAnswer],
    mut ask: Option<&mut Ask<'_>>,
) -> Vec<Graded> {
    all_questions(exam)
        .into_iter()
        .zip(keys)
        .enumerate()
        .map(|(i, (q, key))| {
            let ans = answers.get(i).cloned().unwrap_or_else(|| blank_answer(q));
            let answer_text = format_answer_text(&ans, &q.options);
            let expected_text = format_expected_text(key, &q.options);
            let possible = u32::from(q.points.0);
            if let Some(obj) = grade_objective(q.format, &ans, key, &q.options, possible) {
                return Graded {
                    format: q.format,
                    topic: q.topic.clone(),
                    prompt: q.prompt.clone(),
                    answer: answer_text,
                    sample_answer: expected_text,
                    points_made: obj
                        .checks
                        .into_iter()
                        .map(|(label, ok)| (label, Some(ok)))
                        .collect(),
                    possible,
                    earned: obj.earned,
                    counted: true,
                    from_model: false,
                    engine: GradingEngine::Objective,
                    caveat: None,
                };
            }
            let raw_text = match &ans {
                ExamAnswer::Text(s) => s.as_str(),
                _ => answer_text.as_str(),
            };
            let judged = grade_answer(
                &q.prompt,
                &key.sample_answer,
                &key.rubric,
                raw_text,
                ask.as_deref_mut(),
            );
            let points_made: Vec<(String, Option<bool>)> = judged
                .criteria
                .iter()
                .map(|c| (c.criterion.clone(), c.checked.then_some(c.met)))
                .collect();
            let engine = if judged.from_model {
                GradingEngine::Model
            } else {
                GradingEngine::WordOverlap
            };
            Graded {
                format: q.format,
                topic: q.topic.clone(),
                prompt: q.prompt.clone(),
                answer: answer_text,
                sample_answer: expected_text,
                points_made,
                possible,
                earned: (possible as f32 * share_met(&judged.criteria)).round() as u32,
                counted: is_counted(&judged.criteria),
                from_model: judged.from_model,
                engine,
                caveat: judged.caveat,
            }
        })
        .collect()
}

/// Points available for a question in the total: none when it is not counted.
pub fn counted_possible(g: &Graded) -> u32 {
    if g.counted { g.possible } else { 0 }
}

/// The exam's total: points, percentage and grade at this pass mark.
pub fn total(graded: &[Graded], pass_mark: f32) -> ExamResult {
    let earned: Vec<u32> = graded.iter().map(|g| g.earned).collect();
    let possible: Vec<u32> = graded.iter().map(counted_possible).collect();
    exam_result(&earned, &possible, pass_mark)
}

/// Points earned and available per topic, in the order topics first appear.
pub fn breakdown(graded: &[Graded]) -> Vec<(String, u32, u32)> {
    let mut out: Vec<(String, u32, u32)> = Vec::new();
    for g in graded {
        match out.iter_mut().find(|(t, _, _)| *t == g.topic) {
            Some(row) => {
                row.1 += g.earned;
                row.2 += counted_possible(g);
            }
            None => out.push((g.topic.clone(), g.earned, counted_possible(g))),
        }
    }
    out
}

/// Points earned and available per item format, in the order formats first appear.
pub fn breakdown_by_format(graded: &[Graded]) -> Vec<(ExamItemFormat, u32, u32)> {
    let mut out: Vec<(ExamItemFormat, u32, u32)> = Vec::new();
    for g in graded {
        match out.iter_mut().find(|(f, _, _)| *f == g.format) {
            Some(row) => {
                row.1 += g.earned;
                row.2 += counted_possible(g);
            }
            None => out.push((g.format, g.earned, counted_possible(g))),
        }
    }
    out
}
