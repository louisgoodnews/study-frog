#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `_stack.toml`: scan, index (view), library and polling.

mod common;

use std::fs;
use std::time::{Duration, SystemTime};

use common::vault::{note, stack, write};
use rusqlite::Connection;
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::StackSettings;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::library::build_library;
use studyfrog_store::migrate::migrate;
use studyfrog_store::poll::{PollOutcome, PollState, poll_changes};
use studyfrog_store::reindex::reindex;
use studyfrog_store::scan::scan_vault;
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

type Row = (
    String,
    Option<String>,
    Option<f64>,
    Option<i64>,
    Option<i64>,
);

fn setting(conn: &Connection, folder: &str) -> Option<Row> {
    let sql = "SELECT uid, scheduler, request_retention, daily_new_limit, daily_review_limit FROM stack_setting WHERE folder = ?1";
    conn.query_row(sql, [folder], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
    })
    .ok()
    .map(
        |(uid, sched, ret, new, _review): (String, _, _, _, Option<i64>)| {
            (uid, sched, ret, new, _review)
        },
    )
}

fn bump(path: &std::path::Path) {
    let f = fs::File::options().write(true).open(path).unwrap();
    f.set_modified(SystemTime::now() + Duration::from_secs(10))
        .unwrap();
}

#[test]
fn settings_flow_from_toml_into_the_view_and_the_library() {
    let dir = common::temp_dir("cfg");
    let r = &dir.path;
    write(r, "deck/_stack.md", &stack(1, "Deck"));
    write(
        r,
        "deck/_stack.toml",
        "# note\nscheduler = \"sm2\"\nrequest_retention = 0.85\ndaily_new_limit = 5\n",
    );
    write(r, "deck/a.md", &note(2, "A", ""));
    write(r, "plain/_stack.md", &stack(3, "Plain"));
    write(r, "orphan/_stack.toml", "daily_new_limit = 99\n");
    let mut conn = db();

    let report = reindex(&mut conn, r).unwrap();
    assert_eq!(
        (report.configs_updated, report.configs_invalid),
        (1, 0),
        "only stack folders count: {report:?}"
    );
    let deck = setting(&conn, "deck").unwrap();
    assert_eq!(
        (
            deck.0,
            deck.1.as_deref(),
            deck.2.map(|x| x as f32),
            deck.3,
            deck.4
        ),
        (
            format_ulid(Uid(1)),
            Some("sm2"),
            Some(0.85_f32),
            Some(5),
            None
        )
    );
    let plain = setting(&conn, "plain").unwrap();
    assert_eq!((plain.1, plain.3), (None, None), "no toml: defaults");
    assert!(
        setting(&conn, "orphan").is_none(),
        "a toml without a _stack.md is not a stack"
    );

    let lib = build_library(&scan_vault(r).unwrap());
    let deck = lib
        .stacks
        .items
        .iter()
        .find(|s| s.meta.uid == Uid(1))
        .unwrap();
    assert_eq!(
        deck.settings,
        StackSettings {
            scheduler: Some(Scheduler::Sm2),
            request_retention: Some(0.85),
            daily_new_limit: Some(5),
            daily_review_limit: None,
            familiarity: None
        }
    );
    assert_eq!(
        reindex(&mut conn, r).unwrap().configs_updated,
        0,
        "unchanged files are not re-read"
    );
}

#[test]
fn editing_breaking_and_deleting_a_settings_file_is_tracked() {
    let dir = common::temp_dir("cfg-edit");
    let r = &dir.path;
    write(r, "deck/_stack.md", &stack(1, "Deck"));
    write(r, "deck/_stack.toml", "daily_new_limit = 5\n");
    let mut conn = db();
    reindex(&mut conn, r).unwrap();

    write(r, "deck/_stack.toml", "daily_new_limit = 7\n");
    bump(&r.join("deck/_stack.toml"));
    assert_eq!(reindex(&mut conn, r).unwrap().configs_updated, 1);
    assert_eq!(setting(&conn, "deck").unwrap().3, Some(7));

    write(
        r,
        "deck/_stack.toml",
        "daily_new_limit = 7\ncolour = \"red\"\n",
    );
    bump(&r.join("deck/_stack.toml"));
    let report = reindex(&mut conn, r).unwrap();
    assert_eq!((report.configs_updated, report.configs_invalid), (1, 1));
    assert_eq!(
        setting(&conn, "deck").unwrap().3,
        None,
        "an invalid file falls back to the defaults"
    );
    let error: String = conn
        .query_row(
            "SELECT error FROM stack_config WHERE folder = 'deck'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(error.contains("colour"), "{error}");
    let scan = scan_vault(r).unwrap();
    assert_eq!(scan.config_errors.len(), 1);
    assert!(
        scan.config_errors[0].0.ends_with("_stack.toml")
            && scan.config_errors[0].1.contains("line 2")
    );

    fs::remove_file(r.join("deck/_stack.toml")).unwrap();
    reindex(&mut conn, r).unwrap();
    let rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM stack_config", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 0);
    assert!(
        setting(&conn, "deck").is_some(),
        "the stack itself is still there"
    );
}

#[test]
fn removing_the_stack_file_removes_its_identity_and_a_poll_notices_toml_changes() {
    let dir = common::temp_dir("cfg-poll");
    let r = &dir.path;
    write(r, "deck/_stack.md", &stack(1, "Deck"));
    write(r, "deck/_stack.toml", "daily_new_limit = 5\n");
    let mut conn = db();
    let later = i64::MAX / 2;
    let (state, first) = poll_changes(&mut conn, r, PollState::default(), later, 0).unwrap();
    assert!(matches!(first, PollOutcome::Reindexed(_)));
    assert_eq!(
        poll_changes(&mut conn, r, state, later, 0).unwrap().1,
        PollOutcome::Unchanged
    );

    write(r, "deck/_stack.toml", "daily_new_limit = 9\n");
    bump(&r.join("deck/_stack.toml"));
    let (_, second) = poll_changes(&mut conn, r, state, later, 0).unwrap();
    let PollOutcome::Reindexed(report) = second else {
        panic!("{second:?}")
    };
    assert_eq!(
        report.configs_updated, 1,
        "a settings-only change is detected by polling"
    );

    fs::remove_file(r.join("deck/_stack.md")).unwrap();
    reindex(&mut conn, r).unwrap();
    assert!(setting(&conn, "deck").is_none());
}
