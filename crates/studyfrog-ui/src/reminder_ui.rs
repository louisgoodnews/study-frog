//! Reminders arriving while the app is open, for calendar entries and homework deadlines alike: a
//! toast each, and a line in the notification list. Pure.

use studyfrog_calendar::homework_reminders::DueHomeworkReminder;
use studyfrog_calendar::reminders::{DueAll, DueReminder};
use studyfrog_core::civil::format_minute;

use crate::effect::Effect;
use crate::modal_types::{Toast, ToastAction};
use crate::notify::NotifyTarget;
use crate::state::{Notice, NoticeKind, Screen, State};

/// How many ticks (about a second each) pass between two looks at the calendar.
pub const CHECK_EVERY: u32 = 30;

fn lead_text(minutes: u32) -> String {
    match minutes {
        0 => "now".to_owned(),
        1..=59 => format!("in {minutes} min"),
        60..=1439 => format!("in {} h", minutes / 60),
        _ => format!("in {} days", minutes / 1440),
    }
}

/// The words of one reminder: "Due: Exam prep, in 1 days, 2027-03-01 09:00".
pub fn reminder_text(r: &DueReminder) -> String {
    let date = studyfrog_core::civil::format_date(r.date);
    let at = format_minute(r.start_minute);
    format!("{} ({}, {date} {at})", r.title, lead_text(r.minutes_before))
}

/// The words of one homework reminder: "Homework: Essay 2 (in 2 h, 2026-12-02 09:00)".
pub fn homework_text(r: &DueHomeworkReminder) -> String {
    let date = studyfrog_core::civil::format_date(r.due.date);
    let at = format_minute(r.due.minute);
    format!("{} ({}, {date} {at})", r.title, lead_text(r.lead_min))
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
