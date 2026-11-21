#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for planner urgency classification, filters, and ordering.

use studyfrog_calendar::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_calendar::homework_query::{HomeworkFilter, Urgency, planner_order, urgency};
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
