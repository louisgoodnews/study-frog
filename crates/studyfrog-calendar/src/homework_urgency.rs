//! Urgency of a homework item relative to the local time. Pure.

use studyfrog_core::civil::{LocalTime, date_to_days};

use crate::homework::{HomeworkItem, HomeworkStatus};

/// How pressing a homework item is right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    /// Completed; no deadline pressure any more.
    Completed,
    /// The deadline has passed and the item is not completed.
    Overdue,
    /// Due on the current local date, still ahead.
    DueToday,
    /// Due on the next local date.
    DueTomorrow,
    /// Due later than tomorrow.
    Later,
}

/// Whether an urgency is worth a warning colour (overdue, due today or tomorrow).
pub fn is_due_soon(urgency: Urgency) -> bool {
    matches!(
        urgency,
        Urgency::Overdue | Urgency::DueToday | Urgency::DueTomorrow
    )
}

/// The urgency of `item` at `now`. A completed item is always [`Urgency::Completed`].
pub fn homework_urgency(item: &HomeworkItem, now: LocalTime) -> Urgency {
    if item.status == HomeworkStatus::Completed {
        return Urgency::Completed;
    }
    let days_until = date_to_days(item.due.date) - date_to_days(now.date);
    match days_until {
        d if d < 0 => Urgency::Overdue,
        0 if item.due.minute < now.minute => Urgency::Overdue,
        0 => Urgency::DueToday,
        1 => Urgency::DueTomorrow,
        _ => Urgency::Later,
    }
}
