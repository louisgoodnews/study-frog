#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for timetable persistence in SQLite store.

use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::timetable::{
    delete_timetable_entry, load_timetable_entries, save_timetable_entry,
};

#[test]
fn saves_loads_and_deletes_timetable_entries() {
    let mut conn = open_memory().unwrap();
    studyfrog_store::migrate::migrate(&mut conn, MIGRATIONS).unwrap();

    let entry1 = TimetableEntry {
        id: Uid(1),
        title: "Mathematics I".to_owned(),
        day_of_week: 0, // Monday
        start_minute: 540,
        duration_min: 90,
        location: Some("Room 101".to_owned()),
        instructor: Some("Prof. Gauss".to_owned()),
        stack_uid: Some(Uid(10)),
    };

    let entry2 = TimetableEntry {
        id: Uid(2),
        title: "Physics I".to_owned(),
        day_of_week: 2, // Wednesday
        start_minute: 660,
        duration_min: 60,
        location: Some("Auditorium Max".to_owned()),
        instructor: None,
        stack_uid: None,
    };

    save_timetable_entry(&conn, &entry1).unwrap();
    save_timetable_entry(&conn, &entry2).unwrap();

    let loaded = load_timetable_entries(&conn).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0], entry1);
    assert_eq!(loaded[1], entry2);

    // Update entry1
    let mut updated = entry1.clone();
    updated.location = Some("Room 102".to_owned());
    save_timetable_entry(&conn, &updated).unwrap();

    let loaded_after_update = load_timetable_entries(&conn).unwrap();
    assert_eq!(loaded_after_update.len(), 2);
    assert_eq!(loaded_after_update[0].location.as_deref(), Some("Room 102"));

    // Delete entry1
    let deleted = delete_timetable_entry(&conn, Uid(1)).unwrap();
    assert!(deleted);
    let loaded_after_delete = load_timetable_entries(&conn).unwrap();
    assert_eq!(loaded_after_delete.len(), 1);
    assert_eq!(loaded_after_delete[0].id, Uid(2));
}
