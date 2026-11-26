//! Pure domain types and validation for the Exam Scheduler (EPIC-076, TASK-0305).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic lives in pure free functions.

use studyfrog_core::civil::{Date, add_days, date_to_days};
use studyfrog_core::uid::Uid;

/// How the preparation window before `exam_date` is determined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrepWindow {
    /// Explicit interval in weeks chosen by the user (at least 1).
    UserWeeks(u16),
    /// StudyFrog-proposed interval in weeks with human-readable factor explanations.
    Proposed {
        /// Recommended preparation window in weeks (at least 1).
        weeks: u16,
        /// Short factor explanations justifying the estimate.
        reasons: Vec<String>,
    },
}

/// Vault material attached to an exam plan.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExamTarget {
    /// Whole stack (and nested sub-stacks) identified by vault-relative folder path.
    Stack(String),
    /// Individual card, question, or note identified by its UID.
    Item(Uid),
}

/// One day's scheduled exam preparation block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamScheduleBlock {
    /// Calendar date of this study block.
    pub date: Date,
    /// Ordered item UIDs scheduled for this day (new/weak items first).
    pub item_uids: Vec<Uid>,
    /// Number of unstudied/new items in this block.
    pub new_count: usize,
    /// Number of review/consolidation items in this block.
    pub review_count: usize,
}

/// Persisted exam preparation plan with linked targets and generated daily blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamPlan {
    /// Unique identifier (`Uid(0)` before persistence).
    pub id: Uid,
    /// Exam title (e.g. "Psychologische Diagnostik Klausur").
    pub title: String,
    /// Target exam date.
    pub exam_date: Date,
    /// Preparation window configuration (user-set or StudyFrog-proposed).
    pub window: PrepWindow,
    /// Associated stacks and/or individual items.
    pub targets: Vec<ExamTarget>,
    /// Generated daily preparation blocks within `[start_date, exam_date - 1 day]`.
    pub blocks: Vec<ExamScheduleBlock>,
    /// Creation timestamp in Unix milliseconds.
    pub created_ms: i64,
    /// Last update timestamp in Unix milliseconds.
    pub updated_ms: i64,
}

/// Number of weeks configured in `window`.
pub fn prep_window_weeks(window: &PrepWindow) -> u16 {
    match window {
        PrepWindow::UserWeeks(w) | PrepWindow::Proposed { weeks: w, .. } => *w,
    }
}

/// Inclusive start date `[max(today, exam_date - weeks * 7), exam_date - 1]` for an exam window.
pub fn prep_window_bounds(today: Date, exam_date: Date, weeks: u16) -> Option<(Date, Date)> {
    let exam_days = date_to_days(exam_date);
    let today_days = date_to_days(today);
    if exam_days <= today_days || weeks == 0 {
        return None;
    }
    let end = add_days(exam_date, -1);
    let raw_start = add_days(exam_date, -i64::from(weeks) * 7);
    let start = if date_to_days(raw_start) < today_days {
        today
    } else {
        raw_start
    };
    Some((start, end))
}

/// Validate an `ExamPlan` against `today`.
pub fn validate_exam_plan(plan: &ExamPlan, today: Date) -> Result<(), String> {
    if plan.title.trim().is_empty() {
        return Err("Exam title cannot be empty.".to_owned());
    }
    if date_to_days(plan.exam_date) <= date_to_days(today) {
        return Err("Exam date must be after today.".to_owned());
    }
    if prep_window_weeks(&plan.window) == 0 {
        return Err("Preparation window must be at least 1 week.".to_owned());
    }
    if plan.targets.is_empty() {
        return Err("Attach at least one stack or item to the exam.".to_owned());
    }
    Ok(())
}
