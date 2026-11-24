#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0288: classroom sessions persist as normalized rows, reload in full across a reopen, list,
//! update and delete atomically (children cascade), and export to Markdown.

mod common;

use common::classroom::sample;
use common::temp_dir;
use rusqlite::Connection;
use studyfrog_core::classroom::AssignmentKind;
use studyfrog_core::uid::Uid;
use studyfrog_store::classroom_store::{
    delete_session, export_session_markdown, list_sessions, load_session, save_session,
    update_session,
};
use studyfrog_store::db::{open_db, open_memory};
use studyfrog_store::migrate::{migrate, schema_version};
use studyfrog_store::schema::MIGRATIONS;

const CHILD_TABLES: [&str; 9] = [
    "classroom_participant",
    "classroom_turn",
    "classroom_assignment",
    "classroom_group",
    "classroom_group_member",
    "classroom_submission",
    "classroom_grade",
    "classroom_source",
    "classroom_rubric",
];

fn migrated(conn: &mut Connection) {
    migrate(conn, MIGRATIONS).unwrap();
}

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn migration_32_creates_the_classroom_tables() {
    let mut conn = open_memory().unwrap();
    migrated(&mut conn);
    assert_eq!(schema_version(&conn).unwrap(), 33);
    for table in ["classroom_session", "classroom_grade", "classroom_rubric"]
        .into_iter()
        .chain(CHILD_TABLES)
    {
        assert_eq!(count(&conn, table), 0, "{table} exists and is empty");
    }
}

#[test]
fn a_full_session_round_trips_through_reopen_without_loss() {
    let dir = temp_dir("classroom-reopen");
    let path = dir.path.join("vault.sqlite");
    let original = sample(0x51, "Cell lecture", 1_000);
    {
        let mut conn = open_db(&path).unwrap();
        migrated(&mut conn);
        save_session(&conn, &original).unwrap();
    }
    let mut conn = open_db(&path).unwrap();
    migrated(&mut conn);
    let loaded = load_session(&conn, Uid(0x51)).unwrap();
    assert_eq!(loaded, Some(original));
}

#[test]
fn list_returns_full_sessions_newest_first() {
    let mut conn = open_memory().unwrap();
    migrated(&mut conn);
    let older = sample(1, "Older", 1_000);
    let newer = sample(2, "Newer", 2_000);
    save_session(&conn, &older).unwrap();
    save_session(&conn, &newer).unwrap();
    assert_eq!(list_sessions(&conn).unwrap(), vec![newer, older]);
}

#[test]
fn update_replaces_the_stored_session_and_refuses_unknown_ids() {
    let mut conn = open_memory().unwrap();
    migrated(&mut conn);
    save_session(&conn, &sample(7, "Draft", 1_000)).unwrap();

    let mut revised = sample(7, "Final", 1_000);
    revised.turns.truncate(1);
    revised.submissions.clear();
    assert!(update_session(&conn, &revised).unwrap());
    assert_eq!(load_session(&conn, Uid(7)).unwrap(), Some(revised));

    let stranger = sample(8, "Unknown", 1_000);
    assert!(!update_session(&conn, &stranger).unwrap());
    assert_eq!(load_session(&conn, Uid(8)).unwrap(), None);
    assert_eq!(count(&conn, "classroom_session"), 1);
}

#[test]
fn delete_cascades_to_every_child_row_and_leaves_other_sessions() {
    let mut conn = open_memory().unwrap();
    migrated(&mut conn);
    save_session(&conn, &sample(2, "Kept", 2_000)).unwrap();
    let kept_counts: Vec<i64> = CHILD_TABLES.iter().map(|t| count(&conn, t)).collect();
    let kept = load_session(&conn, Uid(2)).unwrap();

    save_session(&conn, &sample(1, "Gone", 1_000)).unwrap();
    assert!(delete_session(&conn, Uid(1)).unwrap());
    assert!(!delete_session(&conn, Uid(1)).unwrap());

    assert_eq!(count(&conn, "classroom_session"), 1);
    let after: Vec<i64> = CHILD_TABLES.iter().map(|t| count(&conn, t)).collect();
    assert_eq!(after, kept_counts, "only the deleted session's rows went");
    assert_eq!(load_session(&conn, Uid(2)).unwrap(), kept);
    assert_eq!(load_session(&conn, Uid(1)).unwrap(), None);
}

#[test]
fn a_failed_save_stores_nothing_and_a_duplicate_keeps_the_first() {
    let mut conn = open_memory().unwrap();
    migrated(&mut conn);

    let mut broken = sample(3, "Broken", 1_000);
    broken.assignments[0].kind = AssignmentKind::Solo { assignee: 99 };
    assert!(save_session(&conn, &broken).is_err());
    assert_eq!(count(&conn, "classroom_session"), 0);
    for table in CHILD_TABLES {
        assert_eq!(count(&conn, table), 0, "{table} written by a failed save");
    }

    let first = sample(4, "First", 1_000);
    save_session(&conn, &first).unwrap();
    assert!(save_session(&conn, &sample(4, "Second", 2_000)).is_err());
    assert_eq!(load_session(&conn, Uid(4)).unwrap(), Some(first));
}

#[test]
fn missing_sessions_load_as_none_and_export_as_none() {
    let mut conn = open_memory().unwrap();
    migrated(&mut conn);
    assert_eq!(load_session(&conn, Uid(9)).unwrap(), None);
    assert_eq!(export_session_markdown(&conn, Uid(9)).unwrap(), None);
}
