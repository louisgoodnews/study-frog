//! Homework and assignments: items with due dates, checklists, and a lifecycle. Pure: the caller
//! supplies the current time and persists the result.

use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

/// How much an assignment matters to the student.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HomeworkPriority {
    /// Can wait.
    Low,
    /// The default.
    Medium,
    /// Needs attention first.
    High,
}

/// Where an assignment stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HomeworkStatus {
    /// Not begun.
    NotStarted,
    /// Being worked on.
    InProgress,
    /// Handed in or finished.
    Completed,
}

/// One step of an assignment's checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecklistItem {
    /// What to do, one line.
    pub text: String,
    /// Whether it is ticked.
    pub done: bool,
}

/// An assignment, problem set or reading with a deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeworkItem {
    /// Identity.
    pub id: Uid,
    /// What is due (one line).
    pub title: String,
    /// The course or subject it belongs to, if any (a timetable entry or a stack).
    pub course: Option<Uid>,
    /// When work may start, if known.
    pub start: Option<Date>,
    /// When it is due.
    pub due: LocalTime,
    /// Estimated effort in minutes.
    pub estimate_min: u32,
    /// Priority.
    pub priority: HomeworkPriority,
    /// Lifecycle status.
    pub status: HomeworkStatus,
    /// Ordered steps.
    pub checklist: Vec<ChecklistItem>,
    /// Free text.
    pub notes: String,
}

/// Longest allowed title, in characters.
pub const MAX_TITLE_CHARS: usize = 200;
/// Longest allowed estimate: 100 hours.
pub const MAX_ESTIMATE_MIN: u32 = 6_000;
/// Most checklist steps per item.
pub const MAX_CHECKLIST: usize = 50;

fn one_line(text: &str) -> bool {
    !text.chars().any(|c| c.is_control())
}

/// Validate a homework item before it is stored.
pub fn validate_homework(item: &HomeworkItem) -> Result<(), String> {
    let title = item.title.trim();
    if title.is_empty() || !one_line(&item.title) {
        return Err("the title must be one non-empty line".to_owned());
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(format!(
            "the title must be at most {MAX_TITLE_CHARS} characters"
        ));
    }
    if item.estimate_min == 0 || item.estimate_min > MAX_ESTIMATE_MIN {
        return Err("the estimate must be between 1 minute and 100 hours".to_owned());
    }
    if item.due.minute >= 1440 {
        return Err("the due time must be within the day".to_owned());
    }
    if let Some(start) = item.start
        && start > item.due.date
    {
        return Err("the start date must not be after the due date".to_owned());
    }
    if item.checklist.len() > MAX_CHECKLIST {
        return Err(format!("a checklist holds at most {MAX_CHECKLIST} steps"));
    }
    if item
        .checklist
        .iter()
        .any(|c| c.text.trim().is_empty() || !one_line(&c.text))
    {
        return Err("each checklist step must be one non-empty line".to_owned());
    }
    Ok(())
}

/// The next status when the student advances an item: not started, then in progress, then
/// completed, then back to not started (reopening).
pub fn next_status(status: HomeworkStatus) -> HomeworkStatus {
    match status {
        HomeworkStatus::NotStarted => HomeworkStatus::InProgress,
        HomeworkStatus::InProgress => HomeworkStatus::Completed,
        HomeworkStatus::Completed => HomeworkStatus::NotStarted,
    }
}

/// A copy of the item with the checklist step at `index` ticked or unticked, or `None` when the
/// index is out of range.
pub fn toggle_checklist_step(item: &HomeworkItem, index: usize) -> Option<HomeworkItem> {
    let step = item.checklist.get(index)?;
    let mut next = item.clone();
    next.checklist[index] = ChecklistItem {
        text: step.text.clone(),
        done: !step.done,
    };
    Some(next)
}

/// How many checklist steps are ticked, out of how many.
pub fn checklist_progress(item: &HomeworkItem) -> (usize, usize) {
    let done = item.checklist.iter().filter(|c| c.done).count();
    (done, item.checklist.len())
}

/// Whether the item is still open after its due time.
pub fn is_overdue(item: &HomeworkItem, now: LocalTime) -> bool {
    item.status != HomeworkStatus::Completed && item.due < now
}
