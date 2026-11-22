#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The custom report of a learn mode (STORY-0198): it counts that mode's runs only, and nothing of the built-in ones.

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::learn::{StepRow, insert_learn_run, insert_learn_step, set_last_rating};
use studyfrog_store::learn_report::mode_report;
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn run(conn: &Connection, uid: u128, state: &str) -> String {
    let id = format_ulid(Uid(uid));
    conn.execute(
        "INSERT INTO run (uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again, filter_state, filter_diff_min, filter_diff_max, shuffle)
         VALUES (?1, 'flip', ?2, 1000, 'due', 'practice_only', 0, NULL, 1, 'any', NULL, NULL, 0)",
        params![id, state],
    )
    .unwrap();
    id
}

fn step(conn: &Connection, run: &str, seq: i64, item: &str, node: &str, via: Option<&str>) {
    insert_learn_step(
        conn,
        &StepRow {
            run_uid: run,
            seq,
            item_uid: item,
            node_id: node,
            node_kind: node,
            edge_when: via,
            t_ms: seq,
        },
    )
    .unwrap();
}

#[test]
fn a_report_counts_only_its_mode() {
    let conn = db();
    let a = run(&conn, 1, "finished");
    let b = run(&conn, 2, "aborted");
    let builtin = run(&conn, 3, "finished");
    insert_learn_run(&conn, &a, "quick", 1, "h", "t").unwrap();
    insert_learn_run(&conn, &b, "quick", 1, "h", "t").unwrap();
    let item1 = format_ulid(Uid(10));
    let item2 = format_ulid(Uid(11));
    step(&conn, &a, 0, &item1, "show", None);
    step(&conn, &a, 1, &item1, "rate", None);
    set_last_rating(&conn, &a, &item1, "again").unwrap();
    step(&conn, &a, 2, &item1, "show", Some("again"));
    step(&conn, &b, 0, &item2, "rate", None);
    set_last_rating(&conn, &b, &item2, "good").unwrap();
    // A built-in run with its own steps must not appear under the mode.
    step(&conn, &builtin, 0, &item2, "show", None);

    let r = mode_report(&conn, "quick").unwrap();
    assert_eq!(r.runs, 2);
    assert_eq!(r.finished, 1);
    assert_eq!(r.items, 2);
    assert_eq!(
        r.visits,
        vec![("rate".to_owned(), 2), ("show".to_owned(), 2)]
    );
    assert_eq!(
        r.ratings,
        vec![("again".to_owned(), 1), ("good".to_owned(), 1)]
    );
    assert_eq!(r.branches, vec![("again".to_owned(), 1)]);
    assert_eq!(mode_report(&conn, "other").unwrap().runs, 0);
}
