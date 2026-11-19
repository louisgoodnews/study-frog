//! Homework due-time reminders. Pure: the caller supplies the local time and what has fired. The
//! result is the calendar's own [`DueReminder`], so the existing reminder notification path (toast
//! and notification list) shows homework alongside sessions.

use studyfrog_core::civil::{Date, LocalTime, date_to_days};

use crate::homework::{HomeworkItem, HomeworkStatus};
use crate::reminders::{DueReminder, Fired};

/// The lead times a new homework item gets by default: 24 hours, 2 hours and 15 minutes.
pub const DEFAULT_LEAD_TIMES_MIN: [u32; 3] = [24 * 60, 2 * 60, 15];

/// Minutes since the epoch day 0 of a local date-time.
fn absolute_minute(t: LocalTime) -> i64 {
    date_to_days(t.date) * 1440 + i64::from(t.minute)
}

/// The reminders of one homework item that are due at `now`: each lead time whose trigger
/// (`due` minus the lead) has come, while the deadline has not passed, and that has not fired yet
/// for this due date. Completed items never remind.
pub fn due_homework_reminders(
    items: &[HomeworkItem],
    now: LocalTime,
    fired: &[Fired],
) -> Vec<DueReminder> {
    let now_abs = absolute_minute(now);
    let mut out = Vec::new();
    for item in items {
        if item.status == HomeworkStatus::Completed {
            continue;
        }
        push_item_reminders(&mut out, item, now_abs, fired);
    }
    out
}

fn push_item_reminders(
    out: &mut Vec<DueReminder>,
    item: &HomeworkItem,
    now_abs: i64,
    fired: &[Fired],
) {
    let due_abs = absolute_minute(item.due);
    if now_abs > due_abs {
        return;
    }
    for &lead in &item.reminders {
        let trigger = due_abs - i64::from(lead);
        if trigger <= now_abs && !has_fired(fired, item, item.due.date, lead) {
            out.push(reminder_for(item, lead));
        }
    }
}

fn has_fired(fired: &[Fired], item: &HomeworkItem, date: Date, lead: u32) -> bool {
    fired.contains(&(item.id, date, lead))
}

fn reminder_for(item: &HomeworkItem, lead: u32) -> DueReminder {
    DueReminder {
        session: item.id,
        title: item.title.clone(),
        date: item.due.date,
        start_minute: item.due.minute,
        minutes_before: lead,
    }
}

/// The `(item, due date, lead)` key to record once a homework reminder has been shown.
pub fn homework_fired_key(item: &HomeworkItem, lead: u32) -> Fired {
    (item.id, item.due.date, lead)
}
