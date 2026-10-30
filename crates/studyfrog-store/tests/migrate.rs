#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Schema and migration tests.

mod common;

use rusqlite::Connection;
use studyfrog_store::db::{open_db, open_memory};
use studyfrog_store::error::StoreError;
use studyfrog_store::migrate::{Migration, migrate, schema_version};
use studyfrog_store::schema::MIGRATIONS;

const TABLES: [&str; 10] = [
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
    assert_eq!(migrate(&mut conn, MIGRATIONS).unwrap(), 1);
    assert_eq!(migrate(&mut conn, MIGRATIONS).unwrap(), 1);
    let have = tables(&conn);
    for t in TABLES {
        assert!(have.iter().any(|h| h == t), "missing {t}");
    }
}

#[test]
fn upgrades_from_the_previous_version() {
    const V2: Migration = Migration {
        version: 2,
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
    let both = [MIGRATIONS[0], V2];
    assert_eq!(migrate(&mut conn, &both).unwrap(), 2);
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
            supported: 1
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
