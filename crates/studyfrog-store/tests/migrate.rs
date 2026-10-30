#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Schema and migration tests.

mod common;

use rusqlite::Connection;
use studyfrog_store::db::{open_db, open_memory};
use studyfrog_store::error::StoreError;
use studyfrog_store::migrate::{Migration, migrate, schema_version};
use studyfrog_store::schema::MIGRATIONS;

const TABLES: [&str; 20] = [
    "file",
    "item",
    "link",
    "tag",
    "review",
    "mem_state",
    "sm2_state",
    "run",
    "run_queue",
    "run_event",
    "run_filter_kind",
    "run_filter_stack",
    "run_filter_tag",
    "stack_identity",
    "stack_config",
    "session",
    "session_cancelled",
    "board_column",
    "board_card",
    "achievement_unlock",
];

fn tables(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn fresh_database_gets_every_table_and_is_idempotent() {
    let mut conn = open_memory().unwrap();
    assert_eq!(schema_version(&conn).unwrap(), 0);
    let latest = MIGRATIONS.len() as u32;
    assert_eq!(migrate(&mut conn, MIGRATIONS).unwrap(), latest);
    assert_eq!(migrate(&mut conn, MIGRATIONS).unwrap(), latest);
    let have = tables(&conn);
    for t in TABLES {
        assert!(have.iter().any(|h| h == t), "missing {t}");
    }
}

#[test]
fn upgrades_from_the_previous_version() {
    const V6: Migration = Migration {
        version: 6,
        name: "extra",
        sql: "CREATE TABLE extra (x INTEGER);",
    };
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn.execute(
        "INSERT INTO review (item_uid, rating, at_ms, elapsed_days) VALUES ('A', 3, 1, 0)",
        [],
    )
    .unwrap();
    let mut all = MIGRATIONS.to_vec();
    all.push(V6);
    assert_eq!(migrate(&mut conn, &all).unwrap(), 6);
    let kept: i64 = conn
        .query_row("SELECT COUNT(*) FROM review", [], |r| r.get(0))
        .unwrap();
    assert_eq!(kept, 1);
    assert!(tables(&conn).contains(&"extra".to_owned()));
}

#[test]
fn newer_database_is_refused_untouched() {
    let mut conn = open_memory().unwrap();
    conn.pragma_update(None, "user_version", 99).unwrap();
    let err = migrate(&mut conn, MIGRATIONS).unwrap_err();
    assert_eq!(
        err,
        StoreError::SchemaTooNew {
            found: 99,
            supported: MIGRATIONS.len() as u32,
        }
    );
    assert!(tables(&conn).is_empty());
}

#[test]
fn failed_migration_rolls_back() {
    const BAD: Migration = Migration {
        version: 1,
        name: "bad",
        sql: "CREATE TABLE ok (x); CREATE TABLE ok (x);",
    };
    let mut conn = open_memory().unwrap();
    assert!(migrate(&mut conn, &[BAD]).is_err());
    assert_eq!(schema_version(&conn).unwrap(), 0);
    assert!(tables(&conn).is_empty());
}

#[test]
fn constraints_and_foreign_keys_are_enforced() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert!(
        conn.execute(
            "INSERT INTO review (item_uid, rating, at_ms, elapsed_days) VALUES ('A', 9, 1, 0)",
            []
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "INSERT INTO item (uid, path, kind, title) VALUES ('A', 'missing.md', 'note', 't')",
            []
        )
        .is_err()
    );
    conn.execute(
        "INSERT INTO file (path, size, mtime_ns, hash, uid) VALUES ('a.md', 1, 1, 1, 'A')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO item (uid, path, kind, title) VALUES ('A', 'a.md', 'note', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO review (item_uid, rating, at_ms, elapsed_days) VALUES ('A', 3, 1, 0)",
        [],
    )
    .unwrap();
    conn.execute("DELETE FROM file WHERE path = 'a.md'", [])
        .unwrap();
    let items: i64 = conn
        .query_row("SELECT COUNT(*) FROM item", [], |r| r.get(0))
        .unwrap();
    let reviews: i64 = conn
        .query_row("SELECT COUNT(*) FROM review", [], |r| r.get(0))
        .unwrap();
    assert_eq!((items, reviews), (0, 1), "index cascades, history survives");
}

#[test]
fn file_database_uses_wal() {
    let dir = common::temp_dir("wal");
    let conn = open_db(&dir.path.join("studyfrog.sqlite")).unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}

#[test]
fn v1_database_upgrades_in_place_keeping_runs() {
    let mut conn = open_memory().unwrap();
    assert_eq!(migrate(&mut conn, &MIGRATIONS[..1]).unwrap(), 1);
    conn.execute(
        "INSERT INTO run (uid, mode, state) VALUES ('R1', 'flip', 'finished')",
        [],
    )
    .unwrap();
    assert_eq!(
        migrate(&mut conn, MIGRATIONS).unwrap(),
        MIGRATIONS.len() as u32
    );
    let (order, effect, seed, requeue): (String, String, i64, bool) = conn
        .query_row(
            "SELECT order_name, effect, seed, requeue_again FROM run WHERE uid = 'R1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        (order.as_str(), effect.as_str(), seed, requeue),
        ("due", "update_schedule", 0, true)
    );
    assert!(
        conn.execute("UPDATE run SET order_name = 'bogus'", [])
            .is_err(),
        "CHECK on the new column"
    );
}

#[test]
fn v2_database_upgrades_to_v3_keeping_runs_and_gaining_the_filter_and_view() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, &MIGRATIONS[..2]).unwrap();
    conn.execute("INSERT INTO run (uid, mode, state, order_name) VALUES ('R1', 'flip', 'finished', 'shuffled')", []).unwrap();
    conn.execute("INSERT INTO sm2_state (item_uid, ease, repetitions, interval_days, due_ms) VALUES ('A', 2.5, 1, 1, 5)", []).unwrap();
    assert_eq!(
        migrate(&mut conn, MIGRATIONS).unwrap(),
        MIGRATIONS.len() as u32
    );
    let (order, state): (String, String) = conn
        .query_row(
            "SELECT order_name, filter_state FROM run WHERE uid = 'R1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((order.as_str(), state.as_str()), ("shuffled", "any"));
    let last: i64 = conn
        .query_row(
            "SELECT last_review_ms FROM sm2_state WHERE item_uid = 'A'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(last, 0, "existing SM-2 rows get a default last review");
    let view: i64 = conn
        .query_row("SELECT COUNT(*) FROM stack_setting", [], |r| r.get(0))
        .unwrap();
    assert_eq!(view, 0);
}
