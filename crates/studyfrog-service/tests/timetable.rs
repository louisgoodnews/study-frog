#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for timetable service layer.

use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::uid::Uid;
use studyfrog_service::session::VaultSession;
use studyfrog_service::timetable::{
    delete_timetable_entry_checked, list_timetable_entries, save_timetable_entry_checked,
};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::poll::PollState;
use studyfrog_store::schema::MIGRATIONS;

fn mock_session() -> VaultSession {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    VaultSession {
        root: std::path::PathBuf::from("/mock"),
        conn,
        poll: PollState::default(),
        pdf_pending: 0,
    }
}

#[test]
fn lists_saves_and_deletes_timetable_entries() {
    let mut session = mock_session();

    assert!(list_timetable_entries(&session).unwrap().is_empty());

    let entry = TimetableEntry {
        id: Uid(42),
        title: "Algorithms".to_owned(),
        day_of_week: 1, // Tuesday
        start_minute: 600,
        duration_min: 90,
        location: Some("Audimax".to_owned()),
        instructor: Some("Prof. Knuth".to_owned()),
        stack_uid: None,
    };

    let all = save_timetable_entry_checked(&mut session, entry.clone()).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].title, "Algorithms");

    let remaining = delete_timetable_entry_checked(&mut session, Uid(42)).unwrap();
    assert!(remaining.is_empty());
}
