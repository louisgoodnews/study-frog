//! Lead-time reminders for homework deadlines (STORY-0166). Pure: the caller supplies the current
//! local time and what has already been shown.

use studyfrog_core::civil::LocalTime;
use studyfrog_core::uid::Uid;

use crate::homework::{HomeworkItem, HomeworkStatus};

/// The lead times, in minutes before the due time, at which a reminder is shown: 24 hours, 2 hours,
/// and 15 minutes.
pub const HOMEWORK_LEAD_MIN: [u32; 3] = [24 * 60, 2 * 60, 15];

/// One reminder that should be shown now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueHomeworkReminder {
    /// The homework item.
    pub item: Uid,
    /// Its title.
    pub title: String,
    /// The due time.
    pub due: LocalTime,
    /// The lead time that triggered it, in minutes.
    pub lead_min: u32,
    /// The time left before the due time, in minutes (at most the lead time).
    pub remaining_min: u32,
}

/// What has been shown already: (item, lead time in minutes).
pub type FiredHomework = (Uid, u32);

fn absolute_minute(t: LocalTime) -> i64 {
    studyfrog_core::civil::date_to_days(t.date) * 1440 + i64::from(t.minute)
}

/// Reminders whose lead time has begun, for items still open and not yet due, minus those already
/// shown. Only the most specific lead time that has begun is shown: opening the app an hour before a
/// deadline shows the 2-hour reminder once, not the 24-hour one as well.
pub fn due_homework_reminders(
    items: &[HomeworkItem],
    now: LocalTime,
    fired: &[FiredHomework],
) -> Vec<DueHomeworkReminder> {
    let now_abs = absolute_minute(now);
    let mut out = Vec::new();
    for item in items {
        if item.status == HomeworkStatus::Completed {
            continue;
        }
        let due_abs = absolute_minute(item.due);
        if now_abs >= due_abs {
            continue;
        }
        let remaining = u32::try_from(due_abs - now_abs).unwrap_or(u32::MAX);
        let Some(&lead) = HOMEWORK_LEAD_MIN.iter().filter(|&&l| remaining <= l).min() else {
            continue;
        };
        if fired.contains(&(item.id, lead)) {
            continue;
        }
        out.push(DueHomeworkReminder {
            item: item.id,
            title: item.title.clone(),
            due: item.due,
            lead_min: lead,
            remaining_min: remaining,
        });
    }
    out
}
