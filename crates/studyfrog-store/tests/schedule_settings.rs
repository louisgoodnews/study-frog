#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! SM-2 storage, unified schedules, stack lookups, daily counts and vault settings.

mod common;

use common::vault::{card, stack, write};
use rusqlite::Connection;
use studyfrog_core::memory::{MemState, Sm2State};
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::error::StoreError;
use studyfrog_store::migrate::migrate;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schedule::{
    StoredMem, StoredSm2, insert_review, load_schedules, load_sm2_state, save_mem_state,
    save_sm2_state,
};
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::settings::{DailyCounts, daily_counts, item_homes, stack_limits};
use studyfrog_store::vault_config::read_vault_settings;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn fsrs(due: i64) -> StoredMem {
    StoredMem {
        state: MemState {
            stability: 3.0,
            difficulty: 5.0,
        },
        due: UnixMs(due),
        last_review: UnixMs(due - 100),
    }
}

fn sm2(due: i64) -> StoredSm2 {
    StoredSm2 {
        state: Sm2State {
            ease: 2.36,
            repetitions: 3,
            interval_days: 6,
        },
        due: UnixMs(due),
        last_review: UnixMs(due - 200),
    }
}

#[test]
fn sm2_state_round_trips_and_overwrites() {
    let conn = db();
    assert_eq!(load_sm2_state(&conn, Uid(5)).unwrap(), None);
    save_sm2_state(&conn, Uid(5), &sm2(1000)).unwrap();
    save_sm2_state(&conn, Uid(5), &sm2(2000)).unwrap();
    assert_eq!(load_sm2_state(&conn, Uid(5)).unwrap(), Some(sm2(2000)));
}

#[test]
fn schedules_follow_the_items_scheduler_and_survive_a_switch() {
    let conn = db();
    save_mem_state(&conn, Uid(1), &fsrs(1000)).unwrap(); // FSRS only
    save_sm2_state(&conn, Uid(2), &sm2(2000)).unwrap(); // SM-2 only
    save_mem_state(&conn, Uid(3), &fsrs(3000)).unwrap(); // both
    save_sm2_state(&conn, Uid(3), &sm2(3333)).unwrap();

    let by_fsrs = load_schedules(&conn, |_| Scheduler::Fsrs).unwrap();
    let by_sm2 = load_schedules(&conn, |_| Scheduler::Sm2).unwrap();
    assert_eq!(by_fsrs[&Uid(1)].due, UnixMs(1000));
    assert_eq!(
        by_fsrs[&Uid(2)].due,
        UnixMs(2000),
        "falls back to the other scheduler's due date"
    );
    assert_eq!(
        by_fsrs[&Uid(2)].memory,
        None,
        "SM-2 items have no FSRS state"
    );
    assert_eq!(
        (by_fsrs[&Uid(3)].due, by_sm2[&Uid(3)].due),
        (UnixMs(3000), UnixMs(3333))
    );
    assert_eq!(
        by_sm2[&Uid(3)].memory,
        Some(MemState {
            stability: 3.0,
            difficulty: 5.0
        }),
        "difficulty stays available"
    );
    assert_eq!(
        by_sm2[&Uid(1)].due,
        UnixMs(1000),
        "a switched item stays scheduled"
    );
}

#[test]
fn homes_and_limits_come_from_the_indexed_stack_settings() {
    let dir = common::temp_dir("homes");
    let r = &dir.path;
    write(r, "a/_stack.md", &stack(1, "A"));
    write(
        r,
        "a/_stack.toml",
        "scheduler = \"sm2\"\nrequest_retention = 0.8\ndaily_new_limit = 3\ndaily_review_limit = 50\n",
    );
    write(r, "a/x.md", &card(10, "X", "q", "a"));
    write(r, "b/_stack.md", &stack(2, "B"));
    write(r, "b/y.md", &card(11, "Y", "q", "a"));
    write(r, "loose.md", &card(12, "Z", "q", "a"));
    let mut conn = db();
    reindex(&mut conn, r).unwrap();

    let homes = item_homes(&conn).unwrap();
    assert_eq!(homes[&Uid(10)].stack, Uid(1));
    assert_eq!(homes[&Uid(10)].scheduler, Some(Scheduler::Sm2));
    assert_eq!(homes[&Uid(10)].request_retention, Some(0.8));
    assert_eq!(
        (homes[&Uid(11)].scheduler, homes[&Uid(11)].request_retention),
        (None, None)
    );
    assert!(
        !homes.contains_key(&Uid(12)),
        "inbox items have no stack home"
    );

    let limits = stack_limits(&conn).unwrap();
    assert_eq!(limits.len(), 1);
    assert_eq!(
        (limits[&Uid(1)].new_items, limits[&Uid(1)].reviews),
        (Some(3), Some(50))
    );
}

#[test]
fn daily_counts_use_distinct_items_and_the_first_review_ever() {
    let dir = common::temp_dir("counts");
    let r = &dir.path;
    write(r, "a/_stack.md", &stack(1, "A"));
    for id in [10, 11, 12, 13] {
        write(r, &format!("a/c{id}.md"), &card(id, "C", "q", "a"));
    }
    let mut conn = db();
    reindex(&mut conn, r).unwrap();
    let (start, end) = (1_000_000, 2_000_000);
    let review = |item: u128, at: i64| {
        insert_review(
            &conn,
            &Review {
                item: Uid(item),
                rating: Rating::Good,
                at: UnixMs(at),
                elapsed_days: 0.0,
            },
        )
        .unwrap();
    };
    review(10, start + 5); // new today, rated twice (an Again requeue): counts once
    review(10, start + 900);
    review(11, start - 10); // first review before the window ...
    review(11, start + 50); // ... reviewed again today: a review, not a new item
    review(12, end); // exactly at the end: outside the window
    review(13, start - 1); // before the window

    let counts = daily_counts(&conn, UnixMs(start), UnixMs(end)).unwrap();
    assert_eq!(counts.len(), 1);
    assert_eq!(
        counts[&Uid(1)],
        DailyCounts {
            new_done: 1,
            reviews_done: 1
        }
    );
    assert!(
        daily_counts(&conn, UnixMs(end + 1), UnixMs(end + 2))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn vault_settings_default_parse_and_report_errors_by_file() {
    let dir = common::temp_dir("vault-cfg");
    let r = &dir.path;
    assert_eq!(read_vault_settings(r).unwrap().scheduler, Scheduler::Fsrs);
    write(
        r,
        "studyfrog.toml",
        "[srs]\nscheduler = \"sm2\"\nmaximum_interval_days = 90\n",
    );
    let s = read_vault_settings(r).unwrap();
    assert_eq!((s.scheduler, s.maximum_interval_days), (Scheduler::Sm2, 90));
    write(r, "studyfrog.toml", "[srs]\nscheduler = \"nope\"\n");
    match read_vault_settings(r).unwrap_err() {
        StoreError::Config { path, message } => {
            assert_eq!(path, "studyfrog.toml");
            assert!(
                message.contains("line 2") && message.contains("scheduler"),
                "{message}"
            );
        }
        other => panic!("{other:?}"),
    }
}
