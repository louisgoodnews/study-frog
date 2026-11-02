#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Recorded item views in the index database.

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_core::views::{ViewEvent, ViewKind};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::views::{insert_view, load_views, viewed_items};

fn event(item: u128, kind: ViewKind, at: i64, active_ms: u32) -> ViewEvent {
    ViewEvent {
        item: Uid(item),
        kind,
        at: UnixMs(at),
        active_ms,
        expected_ms: Some(active_ms / 2),
    }
}

#[test]
fn views_are_stored_per_item_in_time_order() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    let a = [
        event(1, ViewKind::View, 200, 5_000),
        event(1, ViewKind::View, 100, 3_000),
        event(1, ViewKind::Edit, 200, 5_000),
    ];
    for e in a.iter().chain(&[event(2, ViewKind::View, 50, 1_500)]) {
        insert_view(&conn, e).unwrap();
    }
    let got = load_views(&conn, Uid(1)).unwrap();
    assert_eq!(
        got.iter().map(|e| (e.at.0, e.kind)).collect::<Vec<_>>(),
        [
            (100, ViewKind::View),
            (200, ViewKind::View),
            (200, ViewKind::Edit)
        ]
    );
    assert_eq!(
        load_views(&conn, Uid(2)).unwrap(),
        [event(2, ViewKind::View, 50, 1_500)]
    );
    assert!(load_views(&conn, Uid(3)).unwrap().is_empty());
    assert_eq!(viewed_items(&conn).unwrap(), [Uid(1), Uid(2)]);
}

#[test]
fn a_negative_time_is_refused_by_the_table() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert!(
        conn.execute(
            "INSERT INTO item_view (item_uid, kind, at_ms, active_ms) VALUES ('x', 'view', 1, -5)",
            []
        )
        .is_err()
    );
    assert!(
        conn.execute(
            "INSERT INTO item_view (item_uid, kind, at_ms, active_ms) VALUES ('x', 'answer', 1, 5)",
            []
        )
        .is_err()
    );
}
