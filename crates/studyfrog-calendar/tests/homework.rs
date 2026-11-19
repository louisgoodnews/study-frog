#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the homework domain model, validation, lifecycle, and sorting.

use proptest::prelude::*;
use studyfrog_calendar::homework::{
    ChecklistItem, HomeworkError, HomeworkItem, HomeworkPriority, HomeworkStatus,
    MAX_ESTIMATED_MINUTES, advance_homework_status, checklist_progress,
    homework_priority_from_name, homework_priority_label, homework_priority_name,
    homework_status_from_name, homework_status_label, homework_status_name, normalize_homework,
    normalize_reminders, set_homework_status, sort_homework, toggle_checklist_item,
    toggle_homework_completed, validate_homework,
};
use studyfrog_calendar::session::{MAX_REMINDER_MIN, MAX_REMINDERS};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

fn sample_item() -> HomeworkItem {
    HomeworkItem {
        id: Uid(10),
        title: "Linear Algebra Sheet 3".to_owned(),
        subject: Some("Math".to_owned()),
        stack_uid: Some(Uid(20)),
        timetable_id: Some(Uid(30)),
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 11,
                day: 25,
            },
            minute: 1080,
        },
        start_date: Some(Date {
            year: 2026,
            month: 11,
            day: 18,
        }),
        estimated_minutes: 90,
        priority: HomeworkPriority::High,
        status: HomeworkStatus::NotStarted,
        checklist: vec![
            ChecklistItem {
                text: "Read chapter 4".to_owned(),
                done: false,
            },
            ChecklistItem {
                text: "Solve problems 1-5".to_owned(),
                done: false,
            },
        ],
        notes: "Focus on eigenvalues.\nShow all steps.".to_owned(),
        reminders: vec![0, 60, 1440],
    }
}

#[test]
fn valid_item_and_enum_helpers_succeed() {
    assert!(validate_homework(&sample_item()).is_ok());
    for p in [
        HomeworkPriority::Low,
        HomeworkPriority::Medium,
        HomeworkPriority::High,
        HomeworkPriority::Urgent,
    ] {
        assert_eq!(
            homework_priority_from_name(homework_priority_name(p)),
            Some(p)
        );
        assert!(!homework_priority_label(p).is_empty());
    }
    assert_eq!(
        homework_priority_from_name(" high "),
        Some(HomeworkPriority::High)
    );
    assert_eq!(homework_priority_from_name("unknown"), None);
    for s in [
        HomeworkStatus::NotStarted,
        HomeworkStatus::InProgress,
        HomeworkStatus::Completed,
    ] {
        assert_eq!(homework_status_from_name(homework_status_name(s)), Some(s));
        assert!(!homework_status_label(s).is_empty());
    }
    assert_eq!(homework_status_from_name("unknown"), None);
}

#[test]
fn invalid_title_subject_and_checklist_are_rejected() {
    let mut blank_title = sample_item();
    blank_title.title = "  \t ".to_owned();
    assert_eq!(
        validate_homework(&blank_title),
        Err(HomeworkError::InvalidTitle)
    );

    let mut newline_title = sample_item();
    newline_title.title = "Line 1\nLine 2".to_owned();
    assert_eq!(
        validate_homework(&newline_title),
        Err(HomeworkError::InvalidTitle)
    );

    let mut bad_subject = sample_item();
    bad_subject.subject = Some("  ".to_owned());
    assert_eq!(
        validate_homework(&bad_subject),
        Err(HomeworkError::InvalidSubject)
    );

    let mut bad_check = sample_item();
    bad_check.checklist[1].text = "Bad\nitem".to_owned();
    assert_eq!(
        validate_homework(&bad_check),
        Err(HomeworkError::InvalidChecklistItem(1))
    );
}

#[test]
fn invalid_dates_estimates_and_reminders_are_rejected() {
    let mut bad_due = sample_item();
    bad_due.due.date = Date {
        year: 2026,
        month: 2,
        day: 30,
    };
    assert_eq!(
        validate_homework(&bad_due),
        Err(HomeworkError::InvalidDueDate)
    );

    let mut bad_min = sample_item();
    bad_min.due.minute = 1440;
    assert_eq!(
        validate_homework(&bad_min),
        Err(HomeworkError::InvalidDueMinute)
    );

    let mut bad_start = sample_item();
    bad_start.start_date = Some(Date {
        year: 2026,
        month: 11,
        day: 26,
    });
    assert_eq!(
        validate_homework(&bad_start),
        Err(HomeworkError::StartAfterDue)
    );

    let mut zero_est = sample_item();
    zero_est.estimated_minutes = 0;
    assert_eq!(
        validate_homework(&zero_est),
        Err(HomeworkError::InvalidEstimate)
    );

    let mut huge_est = sample_item();
    huge_est.estimated_minutes = MAX_ESTIMATED_MINUTES + 1;
    assert_eq!(
        validate_homework(&huge_est),
        Err(HomeworkError::InvalidEstimate)
    );

    let mut too_many = sample_item();
    too_many.reminders = vec![1, 2, 3, 4, 5, 6];
    assert_eq!(
        validate_homework(&too_many),
        Err(HomeworkError::TooManyReminders)
    );

    let mut too_far = sample_item();
    too_far.reminders = vec![MAX_REMINDER_MIN + 1];
    assert_eq!(
        validate_homework(&too_far),
        Err(HomeworkError::ReminderTooFar)
    );
}

#[test]
fn lifecycle_transitions_checklist_and_error_display_work() {
    let item = sample_item();
    let completed = toggle_homework_completed(item.clone());
    assert_eq!(completed.status, HomeworkStatus::Completed);
    let reopened = toggle_homework_completed(completed);
    assert_eq!(reopened.status, HomeworkStatus::NotStarted);

    let checked = toggle_checklist_item(item, 0);
    assert!(checked.checklist[0].done);
    assert_eq!(checked.status, HomeworkStatus::NotStarted);
    assert_eq!(checklist_progress(&checked), (1, 2));

    let oob = toggle_checklist_item(checked.clone(), 99);
    assert_eq!(oob, checked);

    let done_with_check = set_homework_status(checked, HomeworkStatus::Completed);
    assert_eq!(
        toggle_homework_completed(done_with_check).status,
        HomeworkStatus::InProgress
    );

    let s1 = advance_homework_status(sample_item());
    assert_eq!(s1.status, HomeworkStatus::InProgress);
    let s2 = advance_homework_status(s1);
    assert_eq!(s2.status, HomeworkStatus::Completed);
    let s3 = advance_homework_status(s2);
    assert_eq!(s3.status, HomeworkStatus::NotStarted);
    assert!(HomeworkError::InvalidTitle.to_string().contains("title"));
}

#[test]
fn normalization_and_sorting_are_deterministic() {
    let mut item = sample_item();
    item.reminders = vec![1440, 60, 1440, 0, 15, 30, 120];
    assert_eq!(
        normalize_reminders(&item.reminders),
        vec![0, 15, 30, 60, 120]
    );
    let norm = normalize_homework(item);
    assert_eq!(norm.reminders.len(), MAX_REMINDERS);

    let mut a = sample_item();
    a.id = Uid(1);
    a.status = HomeworkStatus::Completed;
    let mut b = sample_item();
    b.id = Uid(2);
    b.priority = HomeworkPriority::Low;
    let mut c = sample_item();
    c.id = Uid(3);
    c.priority = HomeworkPriority::Urgent;
    let mut list = vec![a, b, c];
    sort_homework(&mut list);
    assert_eq!(
        list.iter().map(|x| x.id).collect::<Vec<_>>(),
        vec![Uid(3), Uid(2), Uid(1)]
    );
}

proptest! {
    #[test]
    fn normalize_reminders_always_sorted_unique_and_bounded(xs in prop::collection::vec(0u32..100_000, 0..16)) {
        let out = normalize_reminders(&xs);
        prop_assert!(out.len() <= MAX_REMINDERS);
        prop_assert!(out.windows(2).all(|w| w[0] < w[1]));
    }
}
