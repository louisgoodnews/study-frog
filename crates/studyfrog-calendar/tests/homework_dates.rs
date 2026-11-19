#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Regression tests for 4-digit year range (`0..=9999`) validation in homework dates.

use studyfrog_calendar::homework::{
    HomeworkError, HomeworkItem, HomeworkPriority, HomeworkStatus, validate_homework,
};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

fn item_with_years(due_year: i32, start_year: Option<i32>) -> HomeworkItem {
    HomeworkItem {
        id: Uid(1),
        title: "Year boundary check".to_owned(),
        subject: None,
        stack_uid: None,
        timetable_id: None,
        due: LocalTime {
            date: Date {
                year: due_year,
                month: 6,
                day: 15,
            },
            minute: 600,
        },
        start_date: start_year.map(|year| Date {
            year,
            month: 1,
            day: 1,
        }),
        estimated_minutes: 30,
        priority: HomeworkPriority::Medium,
        status: HomeworkStatus::NotStarted,
        checklist: vec![],
        notes: String::new(),
        reminders: vec![],
    }
}

#[test]
fn out_of_range_years_are_rejected_and_four_digit_bounds_are_accepted() {
    assert_eq!(
        validate_homework(&item_with_years(-1, None)),
        Err(HomeworkError::InvalidDueDate)
    );
    assert_eq!(
        validate_homework(&item_with_years(10_000, None)),
        Err(HomeworkError::InvalidDueDate)
    );
    assert_eq!(
        validate_homework(&item_with_years(2026, Some(-1))),
        Err(HomeworkError::InvalidStartDate)
    );
    assert_eq!(
        validate_homework(&item_with_years(2026, Some(10_000))),
        Err(HomeworkError::InvalidStartDate)
    );

    assert!(validate_homework(&item_with_years(0, Some(0))).is_ok());
    assert!(validate_homework(&item_with_years(9999, Some(9999))).is_ok());
}
