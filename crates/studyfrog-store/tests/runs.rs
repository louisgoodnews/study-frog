#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Run, event and schedule persistence.

mod common;

use rusqlite::Connection;
use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::memory::MemState;
use studyfrog_core::plan::{
    Order, RunPlan, ScheduleEffect, filter_all, plan_shuffled, plan_spaced,
};
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::runs::{
    abort_unfinished_runs, append_events, insert_run, load_events, load_run, set_run_state,
};
use studyfrog_store::schedule::{
    StoredMem, insert_review, load_all_mem_states, load_mem_state, load_reviews, save_mem_state,
};
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn run(uid: u128, plan: RunPlan) -> Run {
    Run {
        uid: Uid(uid),
        mode: RunMode::Answer,
        plan,
        queue: vec![Uid(5), Uid(6), Uid(7)],
        state: RunState::Idle,
        started_at: None,
    }
}

fn ev(run: u128, seq: u32, t_ms: u32, item: Option<u128>, kind: EventKind) -> RunEvent {
    RunEvent {
        run: Uid(run),
        seq,
        t_ms,
        item: item.map(Uid),
        kind,
    }
}

#[test]
fn run_and_plan_round_trip_without_the_filter() {
    let conn = db();
    let plan = RunPlan {
        limit: Some(25),
        order: Order::LowestRetrievability,
        ..plan_shuffled(u64::MAX)
    };
    let mut stored = run(1, plan);
    insert_run(&conn, &stored).unwrap();
    assert_eq!(load_run(&conn, Uid(1)).unwrap(), Some(stored.clone()));

    set_run_state(&conn, Uid(1), RunState::Running, Some(UnixMs(1234))).unwrap();
    stored.state = RunState::Running;
    stored.started_at = Some(UnixMs(1234));
    assert_eq!(load_run(&conn, Uid(1)).unwrap(), Some(stored));
    assert_eq!(load_run(&conn, Uid(99)).unwrap(), None);
    assert_eq!(
        load_run(&conn, Uid(1)).unwrap().unwrap().plan.filter,
        filter_all()
    );
}

#[test]
fn events_round_trip_in_order_with_payloads() {
    let conn = db();
    insert_run(&conn, &run(1, plan_spaced())).unwrap();
    let log = vec![
        ev(1, 0, 0, None, EventKind::RunStarted),
        ev(1, 1, 0, Some(5), EventKind::ItemShown),
        ev(
            1,
            2,
            900,
            Some(5),
            EventKind::AnswerSubmitted("Paris \u{1F438}".into()),
        ),
        ev(1, 3, 1200, Some(5), EventKind::ItemRated(Rating::Good)),
        ev(1, 4, 1200, None, EventKind::RunFinished),
    ];
    append_events(&conn, &log).unwrap();
    assert_eq!(load_events(&conn, Uid(1)).unwrap(), log);
    assert!(
        append_events(&conn, &log[..1]).is_err(),
        "(run, seq) is unique"
    );
}

#[test]
fn crash_recovery_aborts_unfinished_runs_and_closes_their_logs() {
    let mut conn = db();
    for (uid, state) in [
        (1, RunState::Running),
        (2, RunState::Paused),
        (3, RunState::Finished),
    ] {
        let mut r = run(uid, plan_spaced());
        r.state = state;
        insert_run(&conn, &r).unwrap();
    }
    append_events(
        &conn,
        &[
            ev(1, 0, 0, None, EventKind::RunStarted),
            ev(1, 1, 40, Some(5), EventKind::ItemShown),
        ],
    )
    .unwrap();

    assert_eq!(abort_unfinished_runs(&mut conn).unwrap(), 2);
    assert_eq!(
        load_run(&conn, Uid(1)).unwrap().unwrap().state,
        RunState::Aborted
    );
    assert_eq!(
        load_run(&conn, Uid(2)).unwrap().unwrap().state,
        RunState::Aborted
    );
    assert_eq!(
        load_run(&conn, Uid(3)).unwrap().unwrap().state,
        RunState::Finished
    );
    let log = load_events(&conn, Uid(1)).unwrap();
    assert_eq!(
        log.last().unwrap(),
        &ev(1, 2, 40, None, EventKind::RunAborted)
    );
    assert_eq!(
        load_events(&conn, Uid(2)).unwrap(),
        [ev(2, 0, 0, None, EventKind::RunAborted)]
    );
    assert_eq!(abort_unfinished_runs(&mut conn).unwrap(), 0, "idempotent");
}

#[test]
fn memory_state_and_reviews_round_trip() {
    let conn = db();
    let item = Uid(5);
    assert_eq!(load_mem_state(&conn, item).unwrap(), None);
    let first = StoredMem {
        state: MemState {
            stability: 2.5,
            difficulty: 6.25,
        },
        due: UnixMs(2000),
        last_review: UnixMs(1000),
    };
    save_mem_state(&conn, item, &first).unwrap();
    let second = StoredMem {
        state: MemState {
            stability: 9.0,
            difficulty: 5.5,
        },
        due: UnixMs(9000),
        last_review: UnixMs(2000),
    };
    save_mem_state(&conn, item, &second).unwrap();
    assert_eq!(load_mem_state(&conn, item).unwrap(), Some(second));
    assert_eq!(load_all_mem_states(&conn).unwrap().len(), 1);

    let reviews = [
        Review {
            item,
            rating: Rating::Good,
            at: UnixMs(1000),
            elapsed_days: 0.0,
        },
        Review {
            item,
            rating: Rating::Again,
            at: UnixMs(2000),
            elapsed_days: 3.0,
        },
    ];
    reviews
        .iter()
        .for_each(|r| insert_review(&conn, r).unwrap());
    assert_eq!(load_reviews(&conn, item).unwrap(), reviews);
    assert_eq!(load_reviews(&conn, Uid(6)).unwrap(), []);
    assert_eq!(plan_spaced().effect, ScheduleEffect::UpdateSchedule);
}

#[test]
fn the_full_filter_round_trips_in_order() {
    use studyfrog_core::plan::{DifficultyRange, ItemFilter, ItemKind, StateFilter};
    let conn = db();
    let filter = ItemFilter {
        kinds: vec![ItemKind::Card, ItemKind::Note],
        stacks: vec![Uid(3), Uid(1)],
        tags_any: vec!["Ünï".into(), "a,b".into(), "x\ty".into()],
        state: StateFilter::NotDue,
        difficulty: Some(DifficultyRange { min: 2.5, max: 7.5 }),
    };
    let stored = run(
        1,
        RunPlan {
            filter,
            ..plan_spaced()
        },
    );
    insert_run(&conn, &stored).unwrap();
    assert_eq!(load_run(&conn, Uid(1)).unwrap(), Some(stored));
}

#[test]
fn the_shuffle_switch_is_stored_with_any_order() {
    let conn = db();
    for (n, (order, shuffle)) in [
        (Order::Due, true),
        (Order::Due, false),
        (Order::HardestFirst, true),
    ]
    .into_iter()
    .enumerate()
    {
        let plan = RunPlan {
            order,
            shuffle,
            ..plan_shuffled(3)
        };
        let stored = run(10 + n as u128, plan);
        insert_run(&conn, &stored).unwrap();
        assert_eq!(load_run(&conn, stored.uid).unwrap(), Some(stored));
    }
}
