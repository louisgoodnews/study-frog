#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for homework V29 schema, SQLite CRUD, atomic TOML persistence, and sync.

mod common;

use std::fs;

use rusqlite::params;
use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::error::StoreError;
use studyfrog_store::homework::{delete_homework, get_homework, list_homework, upsert_homework};
use studyfrog_store::homework_toml::{
    delete_homework_synced, homework_toml_path, read_homework_toml, save_homework_synced,
    sync_homework_from_toml, write_homework_toml,
};
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::vault::list_markdown;

fn hw(id: u128, day: u8, minute: u16, title: &str) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: title.to_owned(),
        subject: Some("Biology".to_owned()),
        stack_uid: Some(Uid(500)),
        timetable_id: Some(Uid(600)),
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 11,
                day,
            },
            minute,
        },
        start_date: Some(Date {
            year: 2026,
            month: 11,
            day: 1,
        }),
        estimated_minutes: 60,
        priority: HomeworkPriority::High,
        status: HomeworkStatus::InProgress,
        checklist: vec![
            ChecklistItem {
                text: "Read notes".to_owned(),
                done: true,
            },
            ChecklistItem {
                text: "Write summary".to_owned(),
                done: false,
            },
        ],
        notes: "Line 1\nLine 2".to_owned(),
        reminders: vec![1440, 60],
    }
}

#[test]
fn v29_migration_and_sqlite_crud_preserve_fired_reminders() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    for table in [
        "homework",
        "homework_checklist",
        "homework_reminder",
        "homework_reminder_fired",
    ] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "missing table {table}");
    }

    let mut a = hw(1, 22, 600, "Later");
    let b = hw(2, 20, 540, "Earlier");
    upsert_homework(&mut conn, &a).unwrap();
    upsert_homework(&mut conn, &b).unwrap();
    conn.execute(
        "INSERT INTO homework_reminder_fired (homework_id, due_date, minutes_before) VALUES (?1, '2026-11-22', 60)",
        [format_ulid(a.id)],
    )
    .unwrap();

    a.title = "Later updated".to_owned();
    a.checklist.pop();
    a.reminders = vec![30];
    upsert_homework(&mut conn, &a).unwrap();

    let fired: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM homework_reminder_fired WHERE homework_id = ?1",
            [format_ulid(a.id)],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(fired, 1, "ON CONFLICT DO UPDATE must keep fired history");

    let all = list_homework(&conn).unwrap();
    assert_eq!(
        all.iter().map(|x| x.id).collect::<Vec<_>>(),
        vec![b.id, a.id]
    );
    assert_eq!(
        get_homework(&conn, a.id).unwrap().unwrap().title,
        "Later updated"
    );
    assert!(delete_homework(&conn, a.id).unwrap());
    assert!(!delete_homework(&conn, a.id).unwrap());

    let fired_after_del: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM homework_reminder_fired WHERE homework_id = ?1",
            [format_ulid(a.id)],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(fired_after_del, 0, "delete cascades fired history");
}

#[test]
fn atomic_toml_write_and_vault_isolation() {
    let dir = common::temp_dir("hw-toml-iso");
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();

    assert!(read_homework_toml(&dir.path).unwrap().is_none());
    let a = hw(10, 25, 720, "Essay");
    let saved = save_homework_synced(&mut conn, &dir.path, &a).unwrap();
    assert_eq!(saved.len(), 1);

    let toml_path = homework_toml_path(&dir.path);
    assert!(toml_path.exists());
    assert!(!dir.path.join(".studyfrog/.homework.toml.tmp").exists());
    assert!(list_markdown(&dir.path).unwrap().is_empty());
}

#[test]
fn toml_sync_and_restore_backfill_work() {
    let dir = common::temp_dir("hw-toml-sync");
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();

    let a = hw(10, 25, 720, "Essay");
    let b = hw(11, 24, 600, "Lab");
    save_homework_synced(&mut conn, &dir.path, &a).unwrap();
    save_homework_synced(&mut conn, &dir.path, &b).unwrap();

    conn.execute(
        "INSERT INTO homework_reminder_fired (homework_id, due_date, minutes_before) VALUES (?1, '2026-11-25', 60)",
        params![format_ulid(a.id)],
    )
    .unwrap();
    let mut a_edited = a.clone();
    a_edited.title = "Essay Revised".to_owned();
    write_homework_toml(&dir.path, &[a_edited]).unwrap();

    assert_eq!(sync_homework_from_toml(&mut conn, &dir.path).unwrap(), 1);
    assert_eq!(
        get_homework(&conn, a.id).unwrap().unwrap().title,
        "Essay Revised"
    );
    assert!(get_homework(&conn, b.id).unwrap().is_none());
    let fired: i64 = conn
        .query_row("SELECT COUNT(*) FROM homework_reminder_fired", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(fired, 1);

    let toml_path = homework_toml_path(&dir.path);
    fs::remove_file(&toml_path).unwrap();
    assert_eq!(sync_homework_from_toml(&mut conn, &dir.path).unwrap(), 1);
    assert!(toml_path.exists());
    assert_eq!(read_homework_toml(&dir.path).unwrap().unwrap().len(), 1);

    assert!(
        delete_homework_synced(&mut conn, &dir.path, b.id)
            .unwrap()
            .is_none()
    );
    let rem = delete_homework_synced(&mut conn, &dir.path, a.id)
        .unwrap()
        .unwrap();
    assert!(rem.is_empty());
    assert!(read_homework_toml(&dir.path).unwrap().unwrap().is_empty());
}

#[test]
fn case_variant_duplicate_ulid_in_toml_is_rejected_without_touching_sqlite_or_disk() {
    let dir = common::temp_dir("hw-dup-ulid");
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();

    let existing = hw(77, 25, 720, "Existing");
    upsert_homework(&mut conn, &existing).unwrap();

    let toml_path = homework_toml_path(&dir.path);
    fs::create_dir_all(toml_path.parent().unwrap()).unwrap();
    let bad_toml = "\
[[homework]]
id = \"01K742SG0A1850M2GA1850M2GA\"
title = \"First\"
due_date = \"2026-11-25\"
due_time = \"09:00\"
estimated_minutes = 30
priority = \"medium\"
status = \"not_started\"

[[homework]]
id = \"01k742sg0a1850m2ga1850m2ga\"
title = \"Second\"
due_date = \"2026-11-25\"
due_time = \"10:00\"
estimated_minutes = 30
priority = \"medium\"
status = \"not_started\"
";
    fs::write(&toml_path, bad_toml).unwrap();

    assert!(matches!(
        read_homework_toml(&dir.path),
        Err(StoreError::Config { .. })
    ));
    assert!(matches!(
        sync_homework_from_toml(&mut conn, &dir.path),
        Err(StoreError::Config { .. })
    ));
    assert_eq!(fs::read_to_string(&toml_path).unwrap(), bad_toml);
    let rows = list_homework(&conn).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, existing.id);
}
