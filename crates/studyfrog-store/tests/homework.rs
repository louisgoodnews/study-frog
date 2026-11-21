#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for homework persistence: round trip, replacement of checklists, and deletion.

use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::homework::{delete_homework, load_homework, save_homework};
use studyfrog_store::schema::MIGRATIONS;

fn open_db() -> rusqlite::Connection {
    let mut conn = open_memory().unwrap();
    studyfrog_store::migrate::migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn item(id: u128, due_day: u8, minute: u16) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: format!("Essay {id}"),
        course: Some(Uid(42)),
        start: None,
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 12,
                day: due_day,
            },
            minute,
        },
        estimate_min: 90,
        priority: HomeworkPriority::Medium,
        status: HomeworkStatus::InProgress,
        checklist: vec![ChecklistItem {
            text: "Outline".to_owned(),
            done: true,
        }],
        notes: "bring the textbook".to_owned(),
    }
}

#[test]
fn saves_loads_in_due_order_and_round_trips_fields() {
    let conn = open_db();
    save_homework(&conn, &item(2, 20, 600)).unwrap();
    save_homework(&conn, &item(1, 5, 900)).unwrap();
    let loaded = load_homework(&conn).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0], item(1, 5, 900));
    assert_eq!(loaded[1], item(2, 20, 600));
}

#[test]
fn saving_again_replaces_checklist_steps() {
    let conn = open_db();
    let mut first = item(3, 5, 0);
    first.checklist = vec![
        ChecklistItem {
            text: "a".to_owned(),
            done: false,
        },
        ChecklistItem {
            text: "b".to_owned(),
            done: false,
        },
    ];
    save_homework(&conn, &first).unwrap();
    first.checklist.truncate(1);
    save_homework(&conn, &first).unwrap();
    let loaded = load_homework(&conn).unwrap();
    assert_eq!(loaded[0].checklist.len(), 1);
}

#[test]
fn delete_removes_item_and_steps() {
    let conn = open_db();
    save_homework(&conn, &item(4, 5, 0)).unwrap();
    assert!(delete_homework(&conn, Uid(4)).unwrap());
    assert!(!delete_homework(&conn, Uid(4)).unwrap());
    assert!(load_homework(&conn).unwrap().is_empty());
}
