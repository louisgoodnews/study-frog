#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Custom learn-mode runs (STORY-0198): the built-in readers leave them out; their steps are kept in order and
//! each item's path can be read back.

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_store::achievements::count_finished_runs;
use studyfrog_store::db::open_memory;
use studyfrog_store::home::recent_runs;
use studyfrog_store::learn::{
    BUILTIN_RUNS, StepRow, custom_mode_totals, insert_learn_run, insert_learn_step, item_path,
    learn_run_of, next_learn_seq, set_last_rating,
};
use studyfrog_store::migrate::migrate;
use studyfrog_store::reports::load_run_starts;
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn run_row(conn: &Connection, uid: u128, started: i64) {
    conn.execute(
        "INSERT INTO run (uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again, filter_state, filter_diff_min, filter_diff_max, shuffle)
         VALUES (?1, 'flip', 'finished', ?2, 'due', 'practice_only', 0, NULL, 1, 'any', NULL, NULL, 0)",
        params![studyfrog_md::ulid::format_ulid(Uid(uid)), started],
    )
    .unwrap();
}

#[test]
fn the_built_in_readers_leave_out_custom_runs() {
    let conn = db();
    run_row(&conn, 1, 1_000);
    run_row(&conn, 2, 2_000);
    let custom = studyfrog_md::ulid::format_ulid(Uid(2));
    insert_learn_run(&conn, &custom, "my-mode", 1, "abc", "name = \"my-mode\"\n").unwrap();

    assert_eq!(count_finished_runs(&conn).unwrap(), 1);
    let recent = recent_runs(&conn, 10).unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].uid, Uid(1));
    let starts = load_run_starts(&conn).unwrap();
    assert!(starts.contains_key(&Uid(1)) && !starts.contains_key(&Uid(2)));
    assert!(BUILTIN_RUNS.contains("learn_run"));
}

#[test]
fn a_custom_run_keeps_its_graph_and_its_steps_in_order() {
    let conn = db();
    run_row(&conn, 3, 3_000);
    let run = studyfrog_md::ulid::format_ulid(Uid(3));
    insert_learn_run(&conn, &run, "my-mode", 2, "h", "snapshot").unwrap();
    assert_eq!(
        learn_run_of(&conn, &run).unwrap(),
        Some(("my-mode".into(), "snapshot".into()))
    );
    assert_eq!(learn_run_of(&conn, "missing").unwrap(), None);

    let item = studyfrog_md::ulid::format_ulid(Uid(9));
    let seq = next_learn_seq(&conn, &run).unwrap();
    insert_learn_step(
        &conn,
        &StepRow {
            run_uid: &run,
            seq,
            item_uid: &item,
            node_id: "rate",
            node_kind: "rate",
            edge_when: None,
            t_ms: 10,
        },
    )
    .unwrap();
    set_last_rating(&conn, &run, &item, "again").unwrap();
    let seq = next_learn_seq(&conn, &run).unwrap();
    insert_learn_step(
        &conn,
        &StepRow {
            run_uid: &run,
            seq,
            item_uid: &item,
            node_id: "show",
            node_kind: "present",
            edge_when: Some("again"),
            t_ms: 20,
        },
    )
    .unwrap();
    let path = item_path(&conn, &run, &item).unwrap();
    assert_eq!(path.len(), 2);
    assert_eq!(
        path[0],
        ("rate".into(), "rate".into(), Some("again".into()))
    );
    assert_eq!(path[1].0, "show");
    assert_eq!(
        custom_mode_totals(&conn).unwrap(),
        vec![("my-mode".to_owned(), 1, 2)]
    );
}
