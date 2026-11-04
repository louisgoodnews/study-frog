#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0218: the upgrade to memory per stack. Every old review and state becomes the primary stack's, none is lost.

mod common;

use common::vault::{card, note, stack, write};
use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::{migrate, schema_version};
use studyfrog_store::reindex::reindex;
use studyfrog_store::schedule::{
    StoredMem, load_mem_state, load_mem_state_in, primary_stack, save_mem_state_in,
};
use studyfrog_store::schema::MIGRATIONS;

fn id(n: u128) -> String {
    format_ulid(Uid(n))
}

fn stack_of(conn: &Connection, table: &str, item: u128) -> Vec<String> {
    let sql = format!("SELECT stack_uid FROM {table} WHERE item_uid = ?1");
    let mut stmt = conn.prepare(&sql).unwrap();
    stmt.query_map([id(item)], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// A database at version 20 holding history for an item in a stack, a loose item and an unknown item.
fn old_database(dir: &std::path::Path) -> Connection {
    write(dir, "a/_stack.md", &stack(1, "A"));
    write(dir, "a/c.md", &card(10, "C", "q", "a"));
    write(dir, "loose.md", &note(12, "Loose", "x"));
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, &MIGRATIONS[..20]).unwrap();
    reindex(&mut conn, dir).unwrap();
    for item in [10, 12, 99] {
        conn.execute(
            "INSERT INTO review (item_uid, rating, at_ms, elapsed_days) VALUES (?1, 3, 1000, 1.0)",
            [id(item)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO mem_state VALUES (?1, 5.0, 4.0, 9000, 1000)",
            [id(item)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sm2_state (item_uid, ease, repetitions, interval_days, due_ms, last_review_ms) VALUES (?1, 2.5, 1, 1, 9000, 1000)",
            [id(item)],
        )
        .unwrap();
    }
    conn
}

#[test]
fn old_history_becomes_the_primary_stacks_and_nothing_is_lost() {
    let dir = common::temp_dir("mem-migrate");
    let mut conn = old_database(&dir.path);
    assert_eq!(schema_version(&conn).unwrap(), 20);
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert_eq!(schema_version(&conn).unwrap(), MIGRATIONS.len() as u32);
    for table in ["review", "mem_state", "sm2_state"] {
        assert_eq!(stack_of(&conn, table, 10), [id(1)], "{table}: in its stack");
        assert_eq!(
            stack_of(&conn, table, 12),
            [id(0)],
            "{table}: loose is the Inbox"
        );
        assert_eq!(
            stack_of(&conn, table, 99),
            [id(0)],
            "{table}: unknown is the Inbox"
        );
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 3, "{table}: one row each");
    }
    let mem = load_mem_state(&conn, Uid(10)).unwrap().unwrap();
    assert_eq!((mem.state.stability, mem.due.0), (5.0, 9000));
    assert_eq!(primary_stack(&conn, Uid(10)).unwrap(), Uid(1));
}

#[test]
fn an_item_can_hold_a_state_in_each_of_two_stacks() {
    let dir = common::temp_dir("mem-two");
    let mut conn = old_database(&dir.path);
    migrate(&mut conn, MIGRATIONS).unwrap();
    let other = StoredMem {
        state: studyfrog_core::memory::MemState {
            stability: 21.0,
            difficulty: 3.0,
        },
        due: studyfrog_core::time::UnixMs(77_000),
        last_review: studyfrog_core::time::UnixMs(2000),
    };
    save_mem_state_in(&conn, (Uid(10), Uid(2)), &other).unwrap();
    assert_eq!(
        load_mem_state_in(&conn, (Uid(10), Uid(2))).unwrap(),
        Some(other)
    );
    let first = load_mem_state_in(&conn, (Uid(10), Uid(1)))
        .unwrap()
        .unwrap();
    assert_eq!(first.due.0, 9000, "the first stack's state is untouched");
    let rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM mem_primary WHERE item_uid = ?1",
            [id(10)],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        rows, 1,
        "screens that show one value per item still see one row"
    );
}
