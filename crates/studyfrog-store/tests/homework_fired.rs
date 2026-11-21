#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests that shown homework reminders persist and are removed with their item.

use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::homework::{
    delete_homework, fired_homework_reminders, mark_homework_reminder_fired,
};
use studyfrog_store::schema::MIGRATIONS;

#[test]
fn fired_reminders_are_recorded_once_and_cleared_on_delete() {
    let mut conn = open_memory().unwrap();
    studyfrog_store::migrate::migrate(&mut conn, MIGRATIONS).unwrap();
    mark_homework_reminder_fired(&conn, Uid(5), 120).unwrap();
    mark_homework_reminder_fired(&conn, Uid(5), 120).unwrap();
    assert_eq!(
        fired_homework_reminders(&conn).unwrap(),
        vec![(Uid(5), 120)]
    );
    delete_homework(&conn, Uid(5)).unwrap();
    assert!(fired_homework_reminders(&conn).unwrap().is_empty());
}
