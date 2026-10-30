#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The v7 migration: shuffling became a switch.

use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;

#[test]
fn v6_runs_stored_as_shuffled_become_sequential_with_the_switch_on() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, &MIGRATIONS[..6]).unwrap();
    conn.execute("INSERT INTO run (uid, mode, state, order_name) VALUES ('R1', 'flip', 'finished', 'shuffled')", []).unwrap();
    conn.execute(
        "INSERT INTO run (uid, mode, state, order_name) VALUES ('R2', 'flip', 'finished', 'due')",
        [],
    )
    .unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    let row = |uid: &str| -> (String, bool) {
        conn.query_row(
            "SELECT order_name, shuffle FROM run WHERE uid = ?1",
            [uid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    };
    assert_eq!(row("R1"), ("sequential".to_owned(), true));
    assert_eq!(row("R2"), ("due".to_owned(), false));
}
