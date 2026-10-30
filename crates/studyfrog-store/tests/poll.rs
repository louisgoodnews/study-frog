#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Polling: coalescing, settling and no-op polls.

mod common;

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use common::vault::{note, write};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::poll::{PollOutcome, PollState, poll_changes};
use studyfrog_store::schema::MIGRATIONS;

const SETTLE: i64 = 200_000_000; // 200 ms

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i64
}

fn later() -> i64 {
    now_ns() + 10_000_000_000
}

#[test]
fn burst_is_coalesced_and_quiet_polls_do_nothing() {
    let dir = common::temp_dir("poll");
    let r = &dir.path;
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();

    for n in 1..=5 {
        write(r, &format!("n{n}.md"), &note(n, &format!("N{n}"), ""));
    }
    let (state, outcome) =
        poll_changes(&mut conn, r, PollState::default(), now_ns(), SETTLE).unwrap();
    assert_eq!(outcome, PollOutcome::Settling, "files were just written");
    assert_eq!(state, PollState::default());

    let (state, outcome) = poll_changes(&mut conn, r, state, later(), SETTLE).unwrap();
    let PollOutcome::Reindexed(report) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!((report.files, report.indexed), (5, 5));

    let (state, outcome) = poll_changes(&mut conn, r, state, later(), SETTLE).unwrap();
    assert_eq!(outcome, PollOutcome::Unchanged);

    fs::remove_file(r.join("n3.md")).unwrap();
    write(r, "n6.md", &note(6, "N6", ""));
    let (state, outcome) = poll_changes(&mut conn, r, state, later(), SETTLE).unwrap();
    let PollOutcome::Reindexed(report) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!((report.removed, report.indexed), (1, 1));
    assert_eq!(
        poll_changes(&mut conn, r, state, later(), SETTLE)
            .unwrap()
            .1,
        PollOutcome::Unchanged
    );
}
