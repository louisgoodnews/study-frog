//! Exam mode data and its grade scale (see `docs/specs/exam-mode.md`). Pure.

use crate::plan::{ItemKind, RunPlan, ScheduleEffect};
use crate::uid::Uid;

/// A grade of the combined international and German scale, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExamGrade {
    /// 95 to 100 %: A+ / 1+.
    APlus,
    /// 90 to under 95 %: A / 1.0.
    A,
    /// 85 to under 90 %: A- / 1.3.
    AMinus,
    /// 80 to under 85 %: B+ / 1.7.
    BPlus,
    /// 75 to under 80 %: B / 2.0.
    B,
    /// 70 to under 75 %: B- / 2.3.
    BMinus,
    /// 65 to under 70 %: C / 3.0.
    C,
    /// 60 to under 65 %: D / 4.0, the minimum to pass.
    D,
    /// Under 60 %: F / 5.0, failed.
    F,
}

/// Every grade with its lower bound in tenths of a percent, best first.
pub const GRADE_BANDS: [(ExamGrade, u32); 9] = [
    (ExamGrade::APlus, 950),
    (ExamGrade::A, 900),
    (ExamGrade::AMinus, 850),
    (ExamGrade::BPlus, 800),
    (ExamGrade::B, 750),
    (ExamGrade::BMinus, 700),
    (ExamGrade::C, 650),
    (ExamGrade::D, 600),
    (ExamGrade::F, 0),
];

/// The percentage needed to pass.
pub const PASS_PERCENT: f32 = 60.0;

/// A percentage in whole tenths (`72.5` is 725), clamped to 0..=1000.
pub fn percent_tenths(percent: f32) -> u32 {
    (percent.clamp(0.0, 100.0) * 10.0).round() as u32
}

/// The grade a percentage earns. Compared in tenths, so `64.99999` and `65.0` cannot be confused by rounding.
pub fn exam_grade(percent: f32) -> ExamGrade {
    let t = percent_tenths(percent);
    GRADE_BANDS
        .iter()
        .find(|(_, low)| t >= *low)
        .map_or(ExamGrade::F, |(g, _)| *g)
}

/// True for every grade from D upwards.
pub fn grade_passes(g: ExamGrade) -> bool {
    g != ExamGrade::F
}

/// The international letter, also the persisted name: `A+` ... `F`.
pub fn grade_letter(g: ExamGrade) -> &'static str {
    match g {
        ExamGrade::APlus => "A+",
        ExamGrade::A => "A",
        ExamGrade::AMinus => "A-",
        ExamGrade::BPlus => "B+",
        ExamGrade::B => "B",
        ExamGrade::BMinus => "B-",
        ExamGrade::C => "C",
        ExamGrade::D => "D",
        ExamGrade::F => "F",
    }
}

/// Inverse of [`grade_letter`].
pub fn grade_from_letter(letter: &str) -> Option<ExamGrade> {
    GRADE_BANDS
        .iter()
        .map(|(g, _)| *g)
        .find(|g| grade_letter(*g) == letter)
}

/// The German university grade: `1+` ... `5.0`.
pub fn grade_german(g: ExamGrade) -> &'static str {
    match g {
        ExamGrade::APlus => "1+",
        ExamGrade::A => "1.0",
        ExamGrade::AMinus => "1.3",
        ExamGrade::BPlus => "1.7",
        ExamGrade::B => "2.0",
        ExamGrade::BMinus => "2.3",
        ExamGrade::C => "3.0",
        ExamGrade::D => "4.0",
        ExamGrade::F => "5.0",
    }
}

/// The grade in words, German first.
pub fn grade_words(g: ExamGrade) -> &'static str {
    match g {
        ExamGrade::APlus => "Sehr gut+ (very good)",
        ExamGrade::A | ExamGrade::AMinus => "Sehr gut (very good)",
        ExamGrade::BPlus | ExamGrade::B | ExamGrade::BMinus => "Gut (good)",
        ExamGrade::C => "Befriedigend (satisfactory)",
        ExamGrade::D => "Ausreichend (sufficient)",
        ExamGrade::F => "Nicht bestanden (failed)",
    }
}

/// The badge text, `A- / 1.3`.
pub fn grade_badge(g: ExamGrade) -> String {
    format!("{} / {}", grade_letter(g), grade_german(g))
}

/// The exam percentage: the mean of `scores` (each 0 to 1) over `items`, in percent with one decimal.
/// Items without a score count 0; no items give 0.
pub fn exam_percent(scores: &[f32], items: usize) -> f32 {
    if items == 0 {
        return 0.0;
    }
    let sum: f32 = scores.iter().map(|s| s.clamp(0.0, 1.0)).sum();
    (sum * 1000.0 / items as f32).round() / 10.0
}

/// A graded exam.
#[derive(Debug, Clone, PartialEq)]
pub struct ExamResult {
    /// The run.
    pub run: Uid,
    /// Items in the exam (each once).
    pub items: u32,
    /// Items answered.
    pub answered: u32,
    /// 0 to 100, one decimal.
    pub percent: f32,
    /// The grade.
    pub grade: ExamGrade,
    /// At least 60 %.
    pub passed: bool,
    /// The runtime that compared the answers.
    pub engine: String,
    /// When it was graded, in Unix milliseconds.
    pub graded_at_ms: i64,
}

/// Grade an exam from one optional score per item (in queue order, each item once).
pub fn grade_exam(
    run: Uid,
    scores: &[Option<f32>],
    answered: u32,
    graded_at_ms: i64,
) -> ExamResult {
    let given: Vec<f32> = scores.iter().flatten().copied().collect();
    let percent = exam_percent(&given, scores.len());
    let grade = exam_grade(percent);
    ExamResult {
        run,
        items: scores.len() as u32,
        answered,
        percent,
        grade,
        passed: grade_passes(grade),
        engine: String::new(),
        graded_at_ms,
    }
}

/// The plan of an exam: only cards and questions, practice-only, no requeue.
pub fn exam_plan(plan: RunPlan) -> Result<RunPlan, String> {
    let gradable = |k: &ItemKind| matches!(k, ItemKind::Card | ItemKind::Question);
    let mut filter = plan.filter;
    if filter.kinds.is_empty() {
        filter.kinds = vec![ItemKind::Card, ItemKind::Question];
    } else {
        filter.kinds.retain(gradable);
        if filter.kinds.is_empty() {
            return Err(
                "an exam needs cards or questions (notes have no answer to grade)".to_owned(),
            );
        }
    }
    Ok(RunPlan {
        filter,
        effect: ScheduleEffect::PracticeOnly,
        requeue_again: false,
        ..plan
    })
}
