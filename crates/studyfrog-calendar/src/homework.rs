//! Homework assignments: pure domain types, constants, and enum helpers.

use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

pub use crate::homework_ops::*;

#[path = "homework_reminders.rs"]
mod homework_reminders;
#[path = "homework_urgency.rs"]
mod homework_urgency;

pub use homework_reminders::*;
pub use homework_urgency::*;

/// Maximum allowed duration estimate in minutes (365 days).
pub const MAX_ESTIMATED_MINUTES: u32 = 525_600;

/// Urgency priority of a homework assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum HomeworkPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    #[default]
    Medium,
    /// High priority.
    High,
    /// Urgent priority.
    Urgent,
}

/// Every homework priority, from lowest to highest.
pub const HOMEWORK_PRIORITIES: [HomeworkPriority; 4] = [
    HomeworkPriority::Low,
    HomeworkPriority::Medium,
    HomeworkPriority::High,
    HomeworkPriority::Urgent,
];

/// The canonical stored name for a homework priority.
pub fn homework_priority_name(priority: HomeworkPriority) -> &'static str {
    match priority {
        HomeworkPriority::Low => "low",
        HomeworkPriority::Medium => "medium",
        HomeworkPriority::High => "high",
        HomeworkPriority::Urgent => "urgent",
    }
}

/// Parse a homework priority from its canonical stored name.
pub fn homework_priority_from_name(name: &str) -> Option<HomeworkPriority> {
    let trimmed = name.trim();
    HOMEWORK_PRIORITIES
        .into_iter()
        .find(|p| homework_priority_name(*p) == trimmed)
}

/// Human-facing label for a homework priority.
pub fn homework_priority_label(priority: HomeworkPriority) -> &'static str {
    match priority {
        HomeworkPriority::Low => "Low",
        HomeworkPriority::Medium => "Medium",
        HomeworkPriority::High => "High",
        HomeworkPriority::Urgent => "Urgent",
    }
}

/// Completion lifecycle state of a homework assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum HomeworkStatus {
    /// Work has not begun.
    #[default]
    NotStarted,
    /// Work is underway.
    InProgress,
    /// Work is finished.
    Completed,
}

/// Every homework status in lifecycle order.
pub const HOMEWORK_STATUSES: [HomeworkStatus; 3] = [
    HomeworkStatus::NotStarted,
    HomeworkStatus::InProgress,
    HomeworkStatus::Completed,
];

/// The canonical stored name for a homework status.
pub fn homework_status_name(status: HomeworkStatus) -> &'static str {
    match status {
        HomeworkStatus::NotStarted => "not_started",
        HomeworkStatus::InProgress => "in_progress",
        HomeworkStatus::Completed => "completed",
    }
}

/// Parse a homework status from its canonical stored name.
pub fn homework_status_from_name(name: &str) -> Option<HomeworkStatus> {
    let trimmed = name.trim();
    HOMEWORK_STATUSES
        .into_iter()
        .find(|s| homework_status_name(*s) == trimmed)
}

/// Human-facing label for a homework status.
pub fn homework_status_label(status: HomeworkStatus) -> &'static str {
    match status {
        HomeworkStatus::NotStarted => "Not started",
        HomeworkStatus::InProgress => "In progress",
        HomeworkStatus::Completed => "Completed",
    }
}

/// One sub-task checkable inside a homework assignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ChecklistItem {
    /// Single-line description of the sub-task.
    pub text: String,
    /// Whether this sub-task has been completed.
    pub done: bool,
}

/// A homework assignment tracked in the planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeworkItem {
    /// Stable identity.
    pub id: Uid,
    /// Single-line assignment title.
    pub title: String,
    /// Optional subject or course name.
    pub subject: Option<String>,
    /// Optional linked flashcard/note stack.
    pub stack_uid: Option<Uid>,
    /// Optional linked weekly timetable lecture/lesson entry.
    pub timetable_id: Option<Uid>,
    /// Local due date and minute of day.
    pub due: LocalTime,
    /// Optional planned start date (must not be after `due.date`).
    pub start_date: Option<Date>,
    /// Positive estimated effort in minutes.
    pub estimated_minutes: u32,
    /// Assignment priority.
    pub priority: HomeworkPriority,
    /// Current lifecycle status.
    pub status: HomeworkStatus,
    /// Ordered sub-tasks for this assignment.
    pub checklist: Vec<ChecklistItem>,
    /// Free-form multi-line notes.
    pub notes: String,
    /// Reminder lead times in minutes before `due`.
    pub reminders: Vec<u32>,
}
