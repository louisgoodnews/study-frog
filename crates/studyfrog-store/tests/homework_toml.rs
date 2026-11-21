#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests that homework TOML is deterministic and round-trips every field.

use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_store::homework_toml::{homework_from_toml, homework_to_toml};

fn item(id: u128, day: u8, minute: u16, title: &str) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: title.to_owned(),
        course: Some(Uid(99)),
        start: Some(Date {
            year: 2026,
            month: 11,
            day: 1,
        }),
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 11,
                day,
            },
            minute,
        },
        estimate_min: 45,
        priority: HomeworkPriority::High,
        status: HomeworkStatus::InProgress,
        checklist: vec![
            ChecklistItem {
                text: "Read \"quoted\" text".to_owned(),
                done: true,
            },
            ChecklistItem {
                text: "Submit".to_owned(),
                done: false,
            },
        ],
        notes: "line one\nline two".to_owned(),
    }
}

#[test]
fn round_trips_every_field() {
    let items = vec![item(1, 3, 600, "Essay"), item(2, 9, 60, "Quiz prep")];
    let text = homework_to_toml(&items);
    let back = homework_from_toml(&text).unwrap();
    assert_eq!(back.len(), 2);
    assert_eq!(back[0], items[0]);
    assert_eq!(back[1], items[1]);
}

#[test]
fn output_does_not_depend_on_input_order() {
    let a = vec![item(1, 3, 600, "Essay"), item(2, 9, 60, "Quiz prep")];
    let b = vec![item(2, 9, 60, "Quiz prep"), item(1, 3, 600, "Essay")];
    assert_eq!(homework_to_toml(&a), homework_to_toml(&b));
}

#[test]
fn malformed_text_is_an_error() {
    assert!(homework_from_toml("homework = [ {").is_err());
}
