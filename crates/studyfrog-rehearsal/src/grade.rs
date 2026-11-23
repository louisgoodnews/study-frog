//! German university grades for mock exams (STORY-0187). Pure: a percentage in, a grade out.

/// A German grade, stored in tenths so 1.3 is `13` and 4.0 is `40`. Valid values are the eleven
/// grades 1.0 to 4.0 in steps of the German scale, plus 5.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GradeBadge {
    /// The grade in tenths: 10 means 1.0, 50 means 5.0.
    pub tenths: u8,
}

/// The bands from the best grade down: (lowest percentage, grade in tenths, German word).
const BANDS: [(f32, u8, &str); 11] = [
    (95.0, 10, "Sehr gut"),
    (90.0, 13, "Sehr gut"),
    (85.0, 17, "Gut"),
    (80.0, 20, "Gut"),
    (75.0, 23, "Gut"),
    (70.0, 27, "Befriedigend"),
    (65.0, 30, "Befriedigend"),
    (60.0, 33, "Befriedigend"),
    (55.0, 37, "Ausreichend"),
    (50.0, 40, "Ausreichend"),
    (f32::MIN, 50, "Nicht bestanden"),
];

/// The percentage earned, or `None` when nothing is possible (no points on the exam).
pub fn percentage(earned: u32, possible: u32) -> Option<f32> {
    if possible == 0 {
        return None;
    }
    Some(earned.min(possible) as f32 * 100.0 / possible as f32)
}

/// Convert a percentage to a German grade and its word. Below `pass_mark` the result is always 5.0
/// (not passed), whatever the band; otherwise the fixed bands apply. A pass mark under 50 % passes
/// the percentages between it and 50 % with 4.0 (Ausreichend), so the grade never contradicts the mark.
pub fn percentage_to_german_grade(pct: f32, pass_mark: f32) -> (GradeBadge, &'static str) {
    let pct = if pct.is_finite() { pct } else { 0.0 };
    if pct < pass_mark {
        let (_, tenths, word) = BANDS[BANDS.len() - 1];
        return (GradeBadge { tenths }, word);
    }
    for &(min, tenths, word) in &BANDS[..BANDS.len() - 1] {
        if pct >= min {
            return (GradeBadge { tenths }, word);
        }
    }
    let (_, tenths, word) = BANDS[BANDS.len() - 2];
    (GradeBadge { tenths }, word)
}

/// The grade as shown to the student, for example `1.3` or `5.0`.
pub fn grade_label(badge: GradeBadge) -> String {
    format!("{}.{}", badge.tenths / 10, badge.tenths % 10)
}

/// Whether the grade passes (4.0 or better).
pub fn is_passing(badge: GradeBadge) -> bool {
    badge.tenths <= 40
}

/// One exam's result: points, percentage, and grade.
#[derive(Debug, Clone, PartialEq)]
pub struct ExamResult {
    /// Points earned.
    pub earned: u32,
    /// Points possible.
    pub possible: u32,
    /// Percentage, if anything was possible.
    pub percent: Option<f32>,
    /// The grade and its word.
    pub grade: GradeBadge,
    /// The German word for the grade.
    pub word: &'static str,
}

/// Summarise an exam: sum the points and grade the percentage. Points above the possible total are
/// capped; an exam with nothing possible grades as 5.0.
pub fn exam_result(earned: &[u32], possible: &[u32], pass_mark: f32) -> ExamResult {
    let e: u32 = earned.iter().sum();
    let p: u32 = possible.iter().sum();
    let percent = percentage(e, p);
    let (grade, word) = percentage_to_german_grade(percent.unwrap_or(0.0), pass_mark);
    ExamResult {
        earned: e.min(p),
        possible: p,
        percent,
        grade,
        word,
    }
}
