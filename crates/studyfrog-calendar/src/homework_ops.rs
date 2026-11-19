//! Validation, lifecycle transitions, normalization, and sorting for homework items.

use std::cmp::Reverse;
use std::fmt;

use studyfrog_core::civil::date_new;

use crate::homework::{ChecklistItem, HomeworkItem, HomeworkStatus, MAX_ESTIMATED_MINUTES};
use crate::session::{MAX_REMINDER_MIN, MAX_REMINDERS};

/// Why a homework item is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeworkError {
    /// Title is empty, whitespace-only, or contains control/line-break characters.
    InvalidTitle,
    /// Subject is empty, whitespace-only, or contains control/line-break characters.
    InvalidSubject,
    /// Due date is not a valid calendar date.
    InvalidDueDate,
    /// Due minute of day is outside `0..1440`.
    InvalidDueMinute,
    /// Start date is not a valid calendar date.
    InvalidStartDate,
    /// Start date is strictly after the due date.
    StartAfterDue,
    /// Estimated duration is zero or exceeds [`MAX_ESTIMATED_MINUTES`].
    InvalidEstimate,
    /// Checklist item at the given zero-based index is invalid.
    InvalidChecklistItem(usize),
    /// More than [`MAX_REMINDERS`] reminders were specified.
    TooManyReminders,
    /// A reminder lead time exceeds [`MAX_REMINDER_MIN`].
    ReminderTooFar,
}

/// Human-readable description of a [`HomeworkError`].
pub fn describe_homework_error(error: &HomeworkError) -> String {
    match error {
        HomeworkError::InvalidTitle => "the title must be one non-empty line".to_owned(),
        HomeworkError::InvalidSubject => "the subject must be one non-empty line".to_owned(),
        HomeworkError::InvalidDueDate => "the due date is not a valid calendar date".to_owned(),
        HomeworkError::InvalidDueMinute => "the due time must be within the day".to_owned(),
        HomeworkError::InvalidStartDate => "the start date is not a valid calendar date".to_owned(),
        HomeworkError::StartAfterDue => "the start date cannot be after the due date".to_owned(),
        HomeworkError::InvalidEstimate => {
            format!("the estimated duration must be between 1 and {MAX_ESTIMATED_MINUTES} minutes")
        }
        HomeworkError::InvalidChecklistItem(idx) => {
            format!("checklist item {} must be one non-empty line", idx + 1)
        }
        HomeworkError::TooManyReminders => {
            format!("at most {MAX_REMINDERS} reminders are allowed")
        }
        HomeworkError::ReminderTooFar => {
            "reminders cannot be more than four weeks ahead".to_owned()
        }
    }
}

impl fmt::Display for HomeworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&describe_homework_error(self))
    }
}

impl std::error::Error for HomeworkError {}

fn is_single_line(text: &str) -> bool {
    !text.trim().is_empty()
        && !text
            .chars()
            .any(|c| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
}

fn validate_dates(item: &HomeworkItem) -> Result<(), HomeworkError> {
    if !(0..=9999).contains(&item.due.date.year)
        || date_new(item.due.date.year, item.due.date.month, item.due.date.day).is_none()
    {
        return Err(HomeworkError::InvalidDueDate);
    }
    if item.due.minute >= 1440 {
        return Err(HomeworkError::InvalidDueMinute);
    }
    if let Some(start) = item.start_date {
        if !(0..=9999).contains(&start.year)
            || date_new(start.year, start.month, start.day).is_none()
        {
            return Err(HomeworkError::InvalidStartDate);
        }
        if start > item.due.date {
            return Err(HomeworkError::StartAfterDue);
        }
    }
    Ok(())
}

fn validate_checklist(checklist: &[ChecklistItem]) -> Result<(), HomeworkError> {
    for (idx, entry) in checklist.iter().enumerate() {
        if !is_single_line(&entry.text) {
            return Err(HomeworkError::InvalidChecklistItem(idx));
        }
    }
    Ok(())
}

fn validate_reminders(reminders: &[u32]) -> Result<(), HomeworkError> {
    if reminders.len() > MAX_REMINDERS {
        return Err(HomeworkError::TooManyReminders);
    }
    if reminders.iter().any(|m| *m > MAX_REMINDER_MIN) {
        return Err(HomeworkError::ReminderTooFar);
    }
    Ok(())
}

/// Validate every invariant of a [`HomeworkItem`].
pub fn validate_homework(item: &HomeworkItem) -> Result<(), HomeworkError> {
    if !is_single_line(&item.title) {
        return Err(HomeworkError::InvalidTitle);
    }
    if let Some(subject) = item.subject.as_deref()
        && !is_single_line(subject)
    {
        return Err(HomeworkError::InvalidSubject);
    }
    validate_dates(item)?;
    if !(1..=MAX_ESTIMATED_MINUTES).contains(&item.estimated_minutes) {
        return Err(HomeworkError::InvalidEstimate);
    }
    validate_checklist(&item.checklist)?;
    validate_reminders(&item.reminders)
}

/// Toggle completion status.
///
/// Restores `InProgress` when any checklist item is checked, otherwise `NotStarted`.
pub fn toggle_homework_completed(item: HomeworkItem) -> HomeworkItem {
    let status = if item.status == HomeworkStatus::Completed {
        if item.checklist.iter().any(|c| c.done) {
            HomeworkStatus::InProgress
        } else {
            HomeworkStatus::NotStarted
        }
    } else {
        HomeworkStatus::Completed
    };
    HomeworkItem { status, ..item }
}

/// Cycle status through `NotStarted -> InProgress -> Completed -> NotStarted`.
pub fn advance_homework_status(item: HomeworkItem) -> HomeworkItem {
    let status = match item.status {
        HomeworkStatus::NotStarted => HomeworkStatus::InProgress,
        HomeworkStatus::InProgress => HomeworkStatus::Completed,
        HomeworkStatus::Completed => HomeworkStatus::NotStarted,
    };
    HomeworkItem { status, ..item }
}

/// Set an explicit status on a homework item.
pub fn set_homework_status(item: HomeworkItem, status: HomeworkStatus) -> HomeworkItem {
    HomeworkItem { status, ..item }
}

/// Toggle the `done` state of a checklist item by index; out-of-bounds is a no-op.
pub fn toggle_checklist_item(mut item: HomeworkItem, index: usize) -> HomeworkItem {
    if let Some(entry) = item.checklist.get_mut(index) {
        entry.done = !entry.done;
    }
    item
}

/// Count `(done, total)` checklist entries on a homework item.
pub fn checklist_progress(item: &HomeworkItem) -> (usize, usize) {
    let done = item.checklist.iter().filter(|c| c.done).count();
    (done, item.checklist.len())
}

/// Sort, deduplicate, and cap reminder lead times to [`MAX_REMINDERS`].
pub fn normalize_reminders(reminders: &[u32]) -> Vec<u32> {
    let mut out = reminders.to_vec();
    out.sort_unstable();
    out.dedup();
    out.truncate(MAX_REMINDERS);
    out
}

/// Return a copy of the homework item with normalized reminders.
pub fn normalize_homework(mut item: HomeworkItem) -> HomeworkItem {
    item.reminders = normalize_reminders(&item.reminders);
    item
}

/// Sort homework items deterministically: active first, earliest due, highest priority, then id.
pub fn sort_homework(items: &mut [HomeworkItem]) {
    items.sort_by_key(|item| {
        (
            item.status == HomeworkStatus::Completed,
            item.due.date,
            item.due.minute,
            Reverse(item.priority),
            item.id,
        )
    });
}
