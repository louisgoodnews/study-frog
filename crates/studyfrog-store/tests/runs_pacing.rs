#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The pacing of a run is stored with it, older runs read as free, and a crash keeps it.

use rusqlite::Connection;
use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::pacing::Pacing;
use studyfrog_core::plan::{RunPlan, plan_spaced};
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::runs::{
    abort_unfinished_runs, append_events, insert_run, load_events, load_run,
};
use studyfrog_store::schema::MIGRATIONS;

fn db(upto: usize) -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, &MIGRATIONS[..upto]).unwrap();
    conn
}

fn run(uid: u128, pacing: Pacing) -> Run {
    Run {
        uid: Uid(uid),
        mode: RunMode::Flip,
        plan: RunPlan {
            pacing,
            ..plan_spaced()
        },
        queue: vec![Uid(5), Uid(6)],
        state: RunState::Idle,
        started_at: None,
    }
}

#[test]
fn each_pacing_is_stored_and_read_back_with_its_seconds() {
    let conn = db(MIGRATIONS.len());
    for (i, pacing) in [
        Pacing::Free,
        Pacing::Countdown(900),
        Pacing::Countdown(60),
        Pacing::Countdown(10_800),
        Pacing::CountUp,
    ]
    .into_iter()
    .enumerate()
    {
        let stored = run(i as u128 + 1, pacing);
        insert_run(&conn, &stored).unwrap();
        assert_eq!(
            load_run(&conn, stored.uid).unwrap(),
            Some(stored.clone()),
            "{pacing:?}"
        );
    }
}

#[test]
fn a_run_made_before_pacing_existed_reads_as_free() {
    let before = MIGRATIONS
        .iter()
        .position(|m| m.name == "run pacing")
        .unwrap();
    let mut conn = db(before);
    conn.execute(
        "INSERT INTO run (uid, mode, state, started_at_ms, order_name, effect, seed, run_limit, requeue_again, filter_state, filter_diff_min, filter_diff_max, shuffle)
         VALUES (?1, 'flip', 'finished', NULL, 'due', 'update_schedule', 7, NULL, 1, 'any', NULL, NULL, 0)",
        [studyfrog_md::ulid::format_ulid(Uid(42))],
    )
    .unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    let old = load_run(&conn, Uid(42)).unwrap().unwrap();
    assert_eq!(old.plan.pacing, Pacing::Free);
}

#[test]
fn the_table_refuses_a_countdown_outside_the_limits_and_an_unknown_word() {
    let conn = db(MIGRATIONS.len());
    assert!(insert_run(&conn, &run(1, Pacing::Countdown(59))).is_err());
    assert!(insert_run(&conn, &run(2, Pacing::Countdown(10_801))).is_err());
    insert_run(&conn, &run(3, Pacing::Free)).unwrap();
    assert!(
        conn.execute("UPDATE run SET pacing = 'stopwatch'", [])
            .is_err(),
        "an unknown word"
    );
}

#[test]
fn a_crash_marks_the_run_aborted_and_keeps_its_pacing_and_its_events() {
    let mut conn = db(MIGRATIONS.len());
    for (uid, pacing) in [(1, Pacing::Countdown(300)), (2, Pacing::CountUp)] {
        let mut r = run(uid, pacing);
        r.state = RunState::Running;
        insert_run(&conn, &r).unwrap();
        conn.execute(
            "UPDATE run SET state = 'running' WHERE uid = ?1",
            [studyfrog_md::ulid::format_ulid(r.uid)],
        )
        .unwrap();
        let events = [
            RunEvent {
                run: r.uid,
                seq: 0,
                t_ms: 0,
                item: None,
                kind: EventKind::RunStarted,
            },
            RunEvent {
                run: r.uid,
                seq: 1,
                t_ms: 100,
                item: Some(Uid(5)),
                kind: EventKind::ItemShown,
            },
        ];
        append_events(&conn, &events).unwrap();
    }
    assert_eq!(abort_unfinished_runs(&mut conn).unwrap(), 2);
    let first = load_run(&conn, Uid(1)).unwrap().unwrap();
    assert_eq!(
        (first.state, first.plan.pacing),
        (RunState::Aborted, Pacing::Countdown(300))
    );
    let second = load_run(&conn, Uid(2)).unwrap().unwrap();
    assert_eq!(
        (second.state, second.plan.pacing),
        (RunState::Aborted, Pacing::CountUp)
    );
    assert!(load_events(&conn, Uid(1)).unwrap().len() >= 2);
}

#[test]
fn the_time_up_event_is_stored_and_read_back() {
    let conn = db(MIGRATIONS.len());
    let r = run(1, Pacing::Countdown(120));
    insert_run(&conn, &r).unwrap();
    let events = [
        RunEvent {
            run: r.uid,
            seq: 0,
            t_ms: 0,
            item: None,
            kind: EventKind::RunStarted,
        },
        RunEvent {
            run: r.uid,
            seq: 1,
            t_ms: 120_000,
            item: Some(Uid(5)),
            kind: EventKind::TimeUp,
        },
        RunEvent {
            run: r.uid,
            seq: 2,
            t_ms: 120_000,
            item: None,
            kind: EventKind::RunFinished,
        },
    ];
    append_events(&conn, &events).unwrap();
    assert_eq!(load_events(&conn, r.uid).unwrap(), events);
}
