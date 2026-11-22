#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for planner urgency classification, filters, and ordering.

use studyfrog_calendar::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_calendar::homework_query::{
    HomeworkFilter, HomeworkTab, Urgency, countdown, due_on, most_urgent, planner_order,
    tab_filter, urgency,
};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

fn at(d: u8, h: u16) -> LocalTime {
    LocalTime {
        date: Date {
            year: 2026,
            month: 11,
            day: d,
        },
        minute: h * 60,
    }
}

fn hw(id: u128, when: LocalTime, status: HomeworkStatus, course: Option<u128>) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: format!("HW {id}"),
        course: course.map(Uid),
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
fn urgency_classes() {
    let now = at(10, 12);
    assert_eq!(
        urgency(&hw(1, at(9, 9), HomeworkStatus::NotStarted, None), now),
        Urgency::Overdue
    );
    assert_eq!(
        urgency(&hw(2, at(10, 18), HomeworkStatus::NotStarted, None), now),
        Urgency::DueToday
    );
    assert_eq!(
        urgency(&hw(3, at(14, 9), HomeworkStatus::NotStarted, None), now),
        Urgency::Upcoming
    );
    assert_eq!(
        urgency(&hw(4, at(30, 9), HomeworkStatus::NotStarted, None), now),
        Urgency::Later
    );
    assert_eq!(
        urgency(&hw(5, at(9, 9), HomeworkStatus::Completed, None), now),
        Urgency::Completed
    );
}

#[test]
fn planner_puts_overdue_first_and_hides_completed_by_default() {
    let now = at(10, 12);
    let items = [
        hw(1, at(30, 9), HomeworkStatus::NotStarted, None),
        hw(2, at(10, 18), HomeworkStatus::NotStarted, None),
        hw(3, at(9, 9), HomeworkStatus::NotStarted, None),
        hw(4, at(9, 8), HomeworkStatus::Completed, None),
    ];
    let order: Vec<u128> = planner_order(&items, &HomeworkFilter::default(), now)
        .iter()
        .map(|i| i.id.0)
        .collect();
    assert_eq!(order, vec![3, 2, 1]);
    let with_done = HomeworkFilter {
        show_completed: true,
        ..Default::default()
    };
    assert_eq!(planner_order(&items, &with_done, now).len(), 4);
}

#[test]
fn filter_by_course_and_urgency() {
    let now = at(10, 12);
    let items = [
        hw(1, at(9, 9), HomeworkStatus::NotStarted, Some(7)),
        hw(2, at(10, 18), HomeworkStatus::NotStarted, Some(8)),
    ];
    let course = HomeworkFilter {
        course: Some(Uid(8)),
        ..Default::default()
    };
    assert_eq!(planner_order(&items, &course, now)[0].id, Uid(2));
    let overdue = HomeworkFilter {
        urgency: Some(Urgency::Overdue),
        ..Default::default()
    };
    assert_eq!(planner_order(&items, &overdue, now)[0].id, Uid(1));
}

#[test]
fn due_on_lists_one_day_earliest_first_and_most_urgent_wins() {
    let now = at(10, 12);
    let items = [
        hw(1, at(10, 18), HomeworkStatus::NotStarted, None),
        hw(2, at(10, 9), HomeworkStatus::NotStarted, None),
        hw(3, at(11, 9), HomeworkStatus::NotStarted, None),
    ];
    let day = at(10, 0).date;
    let due: Vec<u128> = due_on(&items, day).iter().map(|i| i.id.0).collect();
    assert_eq!(due, vec![2, 1]);
    // Item 2 (09:00) is already past at 12:00, so the day's most pressing item is overdue.
    assert_eq!(
        most_urgent(&due_on(&items, day), now),
        Some(Urgency::Overdue)
    );
    assert_eq!(most_urgent(&[], now), None);
    let late = [hw(4, at(9, 9), HomeworkStatus::NotStarted, None)];
    let overdue_day = at(9, 0).date;
    assert_eq!(
        most_urgent(&due_on(&late, overdue_day), now),
        Some(Urgency::Overdue)
    );
}

#[test]
fn tabs_show_the_expected_items() {
    let now = at(10, 12);
    let items = [
        hw(1, at(10, 18), HomeworkStatus::NotStarted, None),
        hw(2, at(9, 9), HomeworkStatus::InProgress, None),
        hw(3, at(12, 9), HomeworkStatus::Completed, None),
        hw(4, at(20, 9), HomeworkStatus::NotStarted, None),
    ];
    let ids = |tab| -> Vec<u128> {
        planner_order(&items, &tab_filter(tab, None), now)
            .iter()
            .map(|i| i.id.0)
            .collect()
    };
    assert_eq!(ids(HomeworkTab::All).len(), 4);
    assert_eq!(ids(HomeworkTab::Active), vec![2, 1, 4]);
    assert_eq!(ids(HomeworkTab::DueToday), vec![1]);
    assert_eq!(ids(HomeworkTab::Overdue), vec![2]);
    assert_eq!(ids(HomeworkTab::Completed), vec![3]);
    let course_only = tab_filter(HomeworkTab::Active, Some(Uid(9)));
    assert!(planner_order(&items, &course_only, now).is_empty());
    assert_eq!(HomeworkTab::default(), HomeworkTab::All);
}

#[test]
fn countdown_reads_in_minutes_hours_and_days() {
    let now = at(10, 12);
    let label = |due: LocalTime, status| {
        let mut item = hw(1, due, status, None);
        item.due = due;
        countdown(&item, now)
    };
    let at_minute = |d: u8, minute: u16| LocalTime { minute, ..at(d, 0) };
    assert_eq!(
        label(at_minute(10, 12 * 60 + 30), HomeworkStatus::NotStarted),
        "due in 30 min"
    );
    assert_eq!(
        label(at_minute(10, 15 * 60), HomeworkStatus::NotStarted),
        "due in 3 h"
    );
    assert_eq!(
        label(at_minute(12, 9 * 60), HomeworkStatus::NotStarted),
        "due in 1 d"
    );
    assert_eq!(
        label(at_minute(9, 9 * 60), HomeworkStatus::NotStarted),
        "overdue by 1 d"
    );
    assert_eq!(
        label(at_minute(9, 9 * 60), HomeworkStatus::Completed),
        "done"
    );
}

#[test]
fn subjects_list_each_stack_once_and_skip_items_without_one() {
    use studyfrog_calendar::homework_query::subjects;
    let when = at(20, 9);
    let items = [
        hw(1, when, HomeworkStatus::NotStarted, Some(5)),
        hw(2, when, HomeworkStatus::NotStarted, None),
        hw(3, when, HomeworkStatus::NotStarted, Some(2)),
        hw(4, when, HomeworkStatus::Completed, Some(5)),
    ];
    assert_eq!(subjects(&items), [Uid(2), Uid(5)]);
    assert!(subjects(&[]).is_empty());
}
