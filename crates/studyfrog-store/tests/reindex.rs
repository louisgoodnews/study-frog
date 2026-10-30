#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Incremental reindex behaviour against a real temporary vault and an in-memory database.

mod common;

use std::fs;
use std::time::{Duration, SystemTime};

use common::vault::{card, note, stack, write};
use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::reindex::{ReindexReport, reindex};
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn title_of(conn: &Connection, uid: u128) -> Option<String> {
    let sql = "SELECT title FROM item WHERE uid = ?1";
    conn.query_row(sql, [format_ulid(Uid(uid))], |r| r.get(0))
        .ok()
}

fn path_of(conn: &Connection, uid: u128) -> Option<String> {
    let sql = "SELECT path FROM item WHERE uid = ?1";
    conn.query_row(sql, [format_ulid(Uid(uid))], |r| r.get(0))
        .ok()
}

fn bump_mtime(path: &std::path::Path) {
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_modified(SystemTime::now() + Duration::from_secs(10))
        .unwrap();
}

#[test]
fn cold_then_warm_index() {
    let dir = common::temp_dir("cold");
    let r = &dir.path;
    write(r, "s/_stack.md", &stack(1, "S"));
    write(r, "s/a.md", &note(2, "A", "links [[B]] and [[C#h|x]]\n"));
    write(r, "s/b.md", &card(3, "B", "q", "a"));
    let mut conn = db();

    let cold = reindex(&mut conn, r).unwrap();
    assert_eq!(
        (cold.files, cold.indexed, cold.unchanged, cold.failed),
        (3, 3, 0, 0)
    );
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM item"), 3);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM link"), 2);
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM item WHERE stack_path = 's'"),
        3
    );

    let warm = reindex(&mut conn, r).unwrap();
    assert_eq!(
        warm,
        ReindexReport {
            files: 3,
            unchanged: 3,
            ..Default::default()
        }
    );
}

#[test]
fn edits_reparse_and_touches_do_not() {
    let dir = common::temp_dir("edit");
    let r = &dir.path;
    write(r, "a.md", &note(2, "Before", ""));
    write(r, "b.md", &note(3, "Same", ""));
    let mut conn = db();
    reindex(&mut conn, r).unwrap();

    write(r, "a.md", &note(2, "After!", ""));
    bump_mtime(&r.join("b.md"));
    let report = reindex(&mut conn, r).unwrap();
    assert_eq!(
        (report.indexed, report.touched, report.unchanged),
        (1, 1, 0)
    );
    assert_eq!(title_of(&conn, 2).as_deref(), Some("After!"));
    assert_eq!(
        reindex(&mut conn, r).unwrap().unchanged,
        2,
        "stat data was refreshed"
    );
}

#[test]
fn rename_keeps_history_and_delete_does_not_erase_it() {
    let dir = common::temp_dir("rename");
    let r = &dir.path;
    write(r, "old/a.md", &note(2, "A", ""));
    let mut conn = db();
    reindex(&mut conn, r).unwrap();
    let uid = format_ulid(Uid(2));
    conn.execute(
        "INSERT INTO review (item_uid, rating, at_ms, elapsed_days) VALUES (?1, 3, 1, 0)",
        [&uid],
    )
    .unwrap();

    fs::create_dir_all(r.join("new")).unwrap();
    fs::rename(r.join("old/a.md"), r.join("new/renamed.md")).unwrap();
    let report = reindex(&mut conn, r).unwrap();
    assert_eq!((report.removed, report.indexed), (1, 1));
    assert_eq!(path_of(&conn, 2).as_deref(), Some("new/renamed.md"));

    fs::remove_file(r.join("new/renamed.md")).unwrap();
    reindex(&mut conn, r).unwrap();
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM item"), 0);
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM review"),
        1,
        "history survives"
    );
}

#[test]
fn duplicate_ids_fail_both_files_and_heal_when_one_goes() {
    let dir = common::temp_dir("dup");
    let r = &dir.path;
    write(r, "a.md", &note(9, "A", ""));
    let mut conn = db();
    reindex(&mut conn, r).unwrap();
    assert_eq!(title_of(&conn, 9).as_deref(), Some("A"));

    write(r, "b.md", &note(9, "B", ""));
    let report = reindex(&mut conn, r).unwrap();
    assert_eq!((report.indexed, report.failed), (0, 2), "{report:?}");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM item"), 0);
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM file WHERE error IS NOT NULL"),
        2
    );

    fs::remove_file(r.join("b.md")).unwrap();
    let healed = reindex(&mut conn, r).unwrap();
    assert_eq!((healed.indexed, healed.failed), (1, 0));
    assert_eq!(title_of(&conn, 9).as_deref(), Some("A"));
}

#[test]
fn adding_a_stack_file_restacks_unchanged_notes() {
    let dir = common::temp_dir("restack");
    let r = &dir.path;
    write(r, "f/a.md", &note(2, "A", ""));
    let mut conn = db();
    reindex(&mut conn, r).unwrap();
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM item WHERE stack_path IS NULL"),
        1
    );

    write(r, "f/_stack.md", &stack(1, "F"));
    let report = reindex(&mut conn, r).unwrap();
    assert_eq!((report.indexed, report.restacked), (1, 1));
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM item WHERE stack_path = 'f'"),
        2
    );
}

#[test]
fn corrupt_files_are_recorded_and_recover_when_fixed() {
    let dir = common::temp_dir("corrupt");
    let r = &dir.path;
    write(r, "ok.md", &note(2, "Ok", ""));
    write(r, "bad.md", "---\nid: nope\nkind: note\ntitle: t\n---\n");
    let mut conn = db();
    let first = reindex(&mut conn, r).unwrap();
    assert_eq!((first.indexed, first.failed), (1, 1));
    let error: String = conn
        .query_row("SELECT error FROM file WHERE path = 'bad.md'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(error.contains("id"), "{error}");

    write(r, "bad.md", &note(3, "Fixed", ""));
    let second = reindex(&mut conn, r).unwrap();
    assert_eq!((second.indexed, second.failed), (1, 0));
    assert_eq!(
        count(&conn, "SELECT COUNT(*) FROM file WHERE error IS NOT NULL"),
        0
    );
}

#[test]
fn the_index_can_be_dropped_and_rebuilt() {
    let dir = common::temp_dir("rebuild");
    let r = &dir.path;
    write(r, "a.md", &note(2, "A", "[[B]]"));
    write(r, "b.md", &note(3, "B", "#tag"));
    let mut conn = db();
    reindex(&mut conn, r).unwrap();
    let snapshot = |c: &Connection| {
        let mut s = c
            .prepare("SELECT uid, path, title FROM item ORDER BY uid")
            .unwrap();
        s.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
    };
    let before = snapshot(&conn);
    conn.execute("DELETE FROM file", []).unwrap();
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM item"), 0);
    let rebuilt = reindex(&mut conn, r).unwrap();
    assert_eq!(rebuilt.indexed, 2);
    assert_eq!(snapshot(&conn), before);
}
