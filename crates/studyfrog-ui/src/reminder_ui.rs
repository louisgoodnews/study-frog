//! Reminders arriving while the app is open, for calendar entries and homework deadlines alike: a
//! toast each, and a line in the notification list. Pure.

use studyfrog_calendar::homework_reminders::DueHomeworkReminder;
use studyfrog_calendar::reminders::{DueAll, DueReminder};
use studyfrog_core::civil::format_minute;
use studyfrog_core::count::counted;

use crate::effect::Effect;
use crate::modal_types::{Toast, ToastAction};
use crate::notify::NotifyTarget;
use crate::state::{Notice, NoticeKind, Screen, State};

/// How many ticks (about a second each) pass between two looks at the calendar.
pub const CHECK_EVERY: u32 = 30;

/// The lead of a calendar entry, as the time before it starts: "in 15 min", "in 2 h", "in 1 day".
fn lead_text(minutes: u32) -> String {
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("in {minutes} min"),
        60..=1439 => format!("in {} h", minutes / 60),
        _ => format!("in {}", counted(minutes / 1440, "day", "days")),
    }
}

/// The time left before a due time: "45 min", "1 h 8 min", "11 h", "1 day".
pub fn left_text(minutes: u32) -> String {
    match minutes {
        0..=59 => format!("{minutes} min"),
        60..=1439 => match minutes % 60 {
            0 => format!("{} h", minutes / 60),
            m => format!("{} h {m} min", minutes / 60),
        },
        _ => counted(minutes / 1440, "day", "days"),
    }
}

/// The words of one reminder: "Due: Exam prep (in 1 day, 2027-03-01 09:00)".
pub fn reminder_text(r: &DueReminder) -> String {
    let date = studyfrog_core::civil::format_date(r.date);
    let at = format_minute(r.start_minute);
    format!("{} ({}, {date} {at})", r.title, lead_text(r.minutes_before))
}

/// The words of one homework reminder, saying the time left before the due time, not the lead it
/// fired at: "Homework: Kurzquiz (due in 1 h 8 min, 2026-11-23 22:30)".
pub fn homework_text(r: &DueHomeworkReminder) -> String {
    let date = studyfrog_core::civil::format_date(r.due.date);
    let at = format_minute(r.due.minute);
    let left = left_text(r.remaining_min);
    format!("{} (due in {left}, {date} {at})", r.title)
}

/// Count ticks while a vault is open and ask for a calendar check on the first and every
/// [`CHECK_EVERY`]th one.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    if state.vault.is_none() {
        return (state, Vec::new());
    }
    let due_now = state.reminder_ticks.is_multiple_of(CHECK_EVERY);
    let state = State {
        reminder_ticks: state.reminder_ticks.wrapping_add(1),
        ..state
    };
    (
        state,
        if due_now {
            vec![Effect::CheckReminders]
        } else {
            Vec::new()
        },
    )
}

/// Show what came due: a toast and a notification each. A failure is shown too, so a broken
/// calendar does not fail silently.
pub fn arrived(state: State, result: Result<DueAll, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(all) => {
            let (state, mut effects) = show_sessions(state, &all.sessions);
            let (state, more) = show_homework(state, &all.homework);
            effects.extend(more);
            (state, effects)
        }
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Checking reminders failed: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

fn show_sessions(state: State, list: &[DueReminder]) -> (State, Vec<Effect>) {
    let mut state = state;
    for r in list {
        state.toasts.push(Toast {
            title: "Reminder".to_owned(),
            detail: reminder_text(r),
            action: ToastAction::OpenCalendar,
            expires_ms: None,
        });
    }
    let mut effects = Vec::new();
    for r in list {
        let text = format!("Reminder: {}", reminder_text(r));
        let target = Some(NotifyTarget::Screen(Screen::Calendar));
        let (next, more) = crate::notify::push(state, (NoticeKind::Info, text), target);
        state = next;
        effects.extend(more);
    }
    (state, effects)
}

fn show_homework(state: State, list: &[DueHomeworkReminder]) -> (State, Vec<Effect>) {
    let mut state = state;
    for r in list {
        state.toasts.push(Toast {
            title: "Homework due".to_owned(),
            detail: homework_text(r),
            action: ToastAction::OpenCalendar,
            expires_ms: None,
        });
    }
    let mut effects = Vec::new();
    for r in list {
        let text = format!("Homework: {}", homework_text(r));
        let target = Some(NotifyTarget::Screen(Screen::Calendar));
        let (next, more) = crate::notify::push(state, (NoticeKind::Info, text), target);
        state = next;
        effects.extend(more);
    }
    (state, effects)
}
