#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the homework model: validation, lifecycle, checklists, and overdue detection.

use studyfrog_calendar::homework::{
    ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus, MAX_CHECKLIST,
    checklist_progress, is_overdue, next_status, toggle_checklist_step, validate_homework,
};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

fn day(d: u8) -> Date {
    Date {
        year: 2026,
        month: 11,
        day: d,
    }
}

fn item() -> HomeworkItem {
    HomeworkItem {
        id: Uid(7),
        title: "Problem set 4".to_owned(),
        course: Some(Uid(9)),
        start: Some(day(2)),
        due: LocalTime {
            date: day(10),
            minute: 23 * 60 + 59,
        },
        estimate_min: 120,
        priority: HomeworkPriority::High,
        status: HomeworkStatus::NotStarted,
        checklist: vec![
            ChecklistItem {
                text: "Read chapter 3".to_owned(),
                done: false,
            },
            ChecklistItem {
                text: "Solve exercises".to_owned(),
                done: true,
            },
        ],
        notes: String::new(),
    }
}

#[test]
fn a_well_formed_item_validates() {
    assert!(validate_homework(&item()).is_ok());
}

#[test]
fn blank_or_multiline_titles_are_rejected() {
    let mut blank = item();
    blank.title = "   ".to_owned();
    assert!(validate_homework(&blank).is_err());
    let mut multi = item();
    multi.title = "a\nb".to_owned();
    assert!(validate_homework(&multi).is_err());
}

#[test]
fn the_estimate_must_be_positive_and_bounded() {
    let mut zero = item();
    zero.estimate_min = 0;
    assert!(validate_homework(&zero).is_err());
    let mut huge = item();
    huge.estimate_min = 6_001;
    assert!(validate_homework(&huge).is_err());
}

#[test]
fn start_after_due_is_rejected() {
    let mut bad = item();
    bad.start = Some(day(11));
    assert!(validate_homework(&bad).is_err());
}

#[test]
fn too_many_or_empty_checklist_steps_are_rejected() {
    let mut empty = item();
    empty.checklist.push(ChecklistItem {
        text: " ".to_owned(),
        done: false,
    });
    assert!(validate_homework(&empty).is_err());
    let mut many = item();
    many.checklist = (0..=MAX_CHECKLIST)
        .map(|i| ChecklistItem {
            text: format!("step {i}"),
            done: false,
        })
        .collect();
    assert!(validate_homework(&many).is_err());
}

#[test]
fn status_cycles_through_the_lifecycle() {
    assert_eq!(
        next_status(HomeworkStatus::NotStarted),
        HomeworkStatus::InProgress
    );
    assert_eq!(
        next_status(HomeworkStatus::InProgress),
        HomeworkStatus::Completed
    );
    assert_eq!(
        next_status(HomeworkStatus::Completed),
        HomeworkStatus::NotStarted
    );
}

#[test]
fn toggling_a_step_flips_only_that_step() {
    let original = item();
    let toggled = toggle_checklist_step(&original, 0).unwrap();
    assert!(toggled.checklist[0].done);
    assert!(toggled.checklist[1].done);
    assert_eq!(toggled.checklist[1], original.checklist[1]);
    assert!(toggle_checklist_step(&original, 5).is_none());
}

#[test]
fn progress_counts_ticked_steps() {
    assert_eq!(checklist_progress(&item()), (1, 2));
}

#[test]
fn overdue_only_when_open_and_past_due() {
    let before = LocalTime {
        date: day(10),
        minute: 12 * 60,
    };
    let after = LocalTime {
        date: day(11),
        minute: 0,
    };
    assert!(!is_overdue(&item(), before));
    assert!(is_overdue(&item(), after));
    let mut done = item();
    done.status = HomeworkStatus::Completed;
    assert!(!is_overdue(&done, after));
}
