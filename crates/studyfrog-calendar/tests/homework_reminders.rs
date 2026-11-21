#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for homework lead-time reminders (24 h, 2 h, 15 min) and their de-duplication.

use studyfrog_calendar::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_calendar::homework_reminders::{HOMEWORK_LEAD_MIN, due_homework_reminders};
use studyfrog_calendar::reminders::due_all;
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

fn at(d: u8, h: u16, m: u16) -> LocalTime {
    LocalTime {
        date: Date {
            year: 2026,
            month: 11,
            day: d,
        },
        minute: h * 60 + m,
    }
}

fn due(id: u128, when: LocalTime, status: HomeworkStatus) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: format!("HW {id}"),
        course: None,
        start: None,
        due: when,
        estimate_min: 30,
        priority: HomeworkPriority::Medium,
        status,
        checklist: vec![],
        notes: String::new(),
    }
}

#[test]
fn lead_times_are_24h_2h_and_15m() {
    assert_eq!(HOMEWORK_LEAD_MIN, [1440, 120, 15]);
}

#[test]
fn nothing_is_due_more_than_a_day_ahead() {
    let items = [due(1, at(12, 18, 0), HomeworkStatus::NotStarted)];
    assert!(due_homework_reminders(&items, at(10, 9, 0), &[]).is_empty());
}

#[test]
fn the_24_hour_reminder_fires_once_a_day_ahead() {
    let items = [due(1, at(12, 18, 0), HomeworkStatus::NotStarted)];
    let r = due_homework_reminders(&items, at(11, 18, 30), &[]);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].lead_min, 1440);
}

#[test]
fn only_the_most_specific_begun_lead_is_shown() {
    let items = [due(1, at(12, 18, 0), HomeworkStatus::NotStarted)];
    let r = due_homework_reminders(&items, at(12, 17, 0), &[]);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].lead_min, 120);
    let fifteen = due_homework_reminders(&items, at(12, 17, 50), &[]);
    assert_eq!(fifteen[0].lead_min, 15);
}

#[test]
fn already_shown_reminders_are_not_repeated() {
    let items = [due(1, at(12, 18, 0), HomeworkStatus::NotStarted)];
    let fired = [(Uid(1), 120)];
    assert!(due_homework_reminders(&items, at(12, 17, 0), &fired).is_empty());
    let later = due_homework_reminders(&items, at(12, 17, 50), &fired);
    assert_eq!(later[0].lead_min, 15);
}

#[test]
fn completed_and_past_items_never_remind() {
    let done = [due(1, at(12, 18, 0), HomeworkStatus::Completed)];
    assert!(due_homework_reminders(&done, at(12, 17, 50), &[]).is_empty());
    let past = [due(2, at(12, 18, 0), HomeworkStatus::NotStarted)];
    assert!(due_homework_reminders(&past, at(12, 18, 0), &[]).is_empty());
}

#[test]
fn due_all_reports_homework_alongside_calendar_reminders() {
    let items = [due(1, at(12, 18, 0), HomeworkStatus::NotStarted)];
    let all = due_all(&[], &items, at(11, 18, 30), &[], &[]);
    assert!(all.sessions.is_empty());
    assert_eq!(all.homework.len(), 1);
    assert_eq!(all.homework[0].lead_min, 24 * 60);
    let fired = [(Uid(1), 24 * 60)];
    assert!(
        due_all(&[], &items, at(11, 18, 30), &[], &fired)
            .homework
            .is_empty()
    );
    assert_eq!(
        due_all(&[], &items, at(9, 9, 0), &[], &[]),
        Default::default()
    );
}
