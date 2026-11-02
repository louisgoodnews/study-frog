#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The stored notification list.

use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::notifications::{
    StoredNotification, clear_notifications, delete_notification, insert_notification,
    load_notifications, prune_notifications, set_all_notifications_read, set_notification_read,
};
use studyfrog_store::schema::MIGRATIONS;

fn note(id: i64, at_ms: i64) -> StoredNotification {
    StoredNotification {
        id,
        kind: "info".into(),
        text: format!("n{id}"),
        at_ms,
        read: false,
        target: None,
    }
}

fn db() -> rusqlite::Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn ids(conn: &rusqlite::Connection) -> Vec<i64> {
    load_notifications(conn)
        .unwrap()
        .iter()
        .map(|n| n.id)
        .collect()
}

#[test]
fn notifications_are_stored_read_back_in_time_order_and_replaced_by_id() {
    let conn = db();
    let mut with_target = note(3, 300);
    with_target.target = Some("screen:calendar".into());
    with_target.kind = "error".into();
    for n in [note(2, 200), with_target.clone(), note(1, 100)] {
        insert_notification(&conn, &n).unwrap();
    }
    let all = load_notifications(&conn).unwrap();
    assert_eq!(all.iter().map(|n| n.id).collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(all[2], with_target, "kind, target and text survive");
    insert_notification(
        &conn,
        &StoredNotification {
            text: "again".into(),
            ..note(2, 250)
        },
    )
    .unwrap();
    let replaced = load_notifications(&conn).unwrap();
    assert_eq!(replaced.len(), 3);
    assert_eq!(replaced[1].text, "again");
}

#[test]
fn read_flags_deleting_and_clearing_work_on_one_or_all() {
    let conn = db();
    for i in 1..=4 {
        insert_notification(&conn, &note(i, i * 10)).unwrap();
    }
    set_notification_read(&conn, 2, true).unwrap();
    let read = |c: &rusqlite::Connection| {
        load_notifications(c)
            .unwrap()
            .iter()
            .filter(|n| n.read)
            .map(|n| n.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(read(&conn), [2]);
    set_notification_read(&conn, 2, false).unwrap();
    assert!(read(&conn).is_empty());
    set_all_notifications_read(&conn, true).unwrap();
    assert_eq!(read(&conn), [1, 2, 3, 4]);
    set_all_notifications_read(&conn, false).unwrap();
    delete_notification(&conn, 3).unwrap();
    assert_eq!(ids(&conn), [1, 2, 4]);
    delete_notification(&conn, 99).unwrap();
    clear_notifications(&conn).unwrap();
    assert!(ids(&conn).is_empty());
}

#[test]
fn pruning_drops_what_is_too_old_and_keeps_the_newest_up_to_the_cap() {
    let conn = db();
    for i in 1..=10 {
        insert_notification(&conn, &note(i, i * 100)).unwrap();
    }
    prune_notifications(&conn, 100, 350).unwrap();
    assert_eq!(ids(&conn), [4, 5, 6, 7, 8, 9, 10], "older than 350 is gone");
    prune_notifications(&conn, 3, 0).unwrap();
    assert_eq!(ids(&conn), [8, 9, 10], "only the newest three stay");
    prune_notifications(&conn, 0, 0).unwrap();
    assert!(ids(&conn).is_empty());
}

#[test]
fn the_table_refuses_unknown_kinds_and_read_values() {
    let conn = db();
    let bad_kind = StoredNotification {
        kind: "warning".into(),
        ..note(1, 1)
    };
    assert!(insert_notification(&conn, &bad_kind).is_err());
    assert!(
        conn.execute(
            "INSERT INTO notification (id, kind, text, at_ms, read) VALUES (1, 'info', 'x', 1, 7)",
            []
        )
        .is_err()
    );
}
