#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Due-time reminders (24h, 2h, 15m lead times) and urgency states for homework.

use studyfrog_calendar::homework::{
    DEFAULT_LEAD_TIMES_MIN, HomeworkItem, HomeworkPriority, HomeworkStatus, Urgency,
    due_homework_reminders, homework_fired_key, homework_urgency, is_due_soon,
};
use studyfrog_calendar::reminders::Fired;
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

fn at(year: i32, month: u8, day: u8, minute: u16) -> LocalTime {
    LocalTime {
        date: Date { year, month, day },
        minute,
    }
}

fn item(id: u128, due: LocalTime, reminders: Vec<u32>) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: format!("Essay {id}"),
        subject: None,
        stack_uid: None,
        timetable_id: None,
        due,
        start_date: None,
        estimated_minutes: 60,
        priority: HomeworkPriority::Medium,
        status: HomeworkStatus::NotStarted,
        checklist: Vec::new(),
        notes: String::new(),
        reminders,
    }
}

#[test]
fn default_lead_times_are_24h_2h_and_15m() {
    assert_eq!(DEFAULT_LEAD_TIMES_MIN, [1440, 120, 15]);
}

#[test]
fn reminder_fires_once_its_lead_time_has_come() {
    // Due 2027-03-01 09:00 (minute 540); 2h lead starts at 07:00.
    let due = at(2027, 3, 1, 540);
    let items = [item(1, due, vec![120])];
    assert!(due_homework_reminders(&items, at(2027, 3, 1, 419), &[]).is_empty());
    let due_now = due_homework_reminders(&items, at(2027, 3, 1, 420), &[]);
    assert_eq!(due_now.len(), 1);
    assert_eq!(due_now[0].minutes_before, 120);
    assert_eq!(due_now[0].start_minute, 540);
    assert_eq!(due_now[0].title, "Essay 1");
}

#[test]
fn a_24_hour_lead_crosses_midnight() {
    let due = at(2027, 3, 2, 540);
    let items = [item(2, due, DEFAULT_LEAD_TIMES_MIN.to_vec())];
    let on_day_before = due_homework_reminders(&items, at(2027, 3, 1, 540), &[]);
    assert_eq!(on_day_before.len(), 1);
    assert_eq!(on_day_before[0].minutes_before, 1440);
}

#[test]
fn fired_reminders_are_not_repeated() {
    let due = at(2027, 3, 1, 540);
    let items = [item(3, due, vec![15, 120])];
    let fired: Vec<Fired> = vec![homework_fired_key(&items[0], 15)];
    let now = at(2027, 3, 1, 530);
    let left = due_homework_reminders(&items, now, &fired);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].minutes_before, 120);
}

#[test]
fn missed_reminder_still_shows_until_the_deadline() {
    let due = at(2027, 3, 1, 540);
    let items = [item(4, due, vec![15])];
    assert_eq!(
        due_homework_reminders(&items, at(2027, 3, 1, 539), &[]).len(),
        1
    );
    assert!(due_homework_reminders(&items, at(2027, 3, 1, 541), &[]).is_empty());
}

#[test]
fn completed_items_never_remind() {
    let due = at(2027, 3, 1, 540);
    let mut done = item(5, due, vec![15]);
    done.status = HomeworkStatus::Completed;
    assert!(due_homework_reminders(&[done], at(2027, 3, 1, 530), &[]).is_empty());
}

#[test]
fn urgency_states_follow_the_local_date_and_time() {
    let now = at(2027, 3, 1, 600);
    assert_eq!(
        homework_urgency(&item(6, at(2027, 2, 28, 900), vec![]), now),
        Urgency::Overdue
    );
    assert_eq!(
        homework_urgency(&item(7, at(2027, 3, 1, 500), vec![]), now),
        Urgency::Overdue
    );
    assert_eq!(
        homework_urgency(&item(8, at(2027, 3, 1, 900), vec![]), now),
        Urgency::DueToday
    );
    assert_eq!(
        homework_urgency(&item(9, at(2027, 3, 2, 900), vec![]), now),
        Urgency::DueTomorrow
    );
    assert_eq!(
        homework_urgency(&item(10, at(2027, 3, 3, 900), vec![]), now),
        Urgency::Later
    );
}

#[test]
fn completed_items_are_completed_not_overdue() {
    let mut done = item(11, at(2027, 2, 1, 900), vec![]);
    done.status = HomeworkStatus::Completed;
    let urgency = homework_urgency(&done, at(2027, 3, 1, 600));
    assert_eq!(urgency, Urgency::Completed);
    assert!(!is_due_soon(urgency));
}

#[test]
fn due_soon_covers_overdue_today_and_tomorrow_only() {
    assert!(is_due_soon(Urgency::Overdue));
    assert!(is_due_soon(Urgency::DueToday));
    assert!(is_due_soon(Urgency::DueTomorrow));
    assert!(!is_due_soon(Urgency::Later));
    assert!(!is_due_soon(Urgency::Completed));
}
