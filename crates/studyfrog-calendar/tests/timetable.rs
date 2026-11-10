#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for timetable model, validation, and recurrence expansion.

use studyfrog_calendar::timetable::{
    TimetableEntry, timetable_occurrences, timetable_to_session, validate_timetable_entry,
};
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;

fn entry(dow: u8, start: u16, dur: u16) -> TimetableEntry {
    TimetableEntry {
        id: Uid(101),
        title: "Calculus I".to_owned(),
        day_of_week: dow,
        start_minute: start,
        duration_min: dur,
        location: Some("Hall B".to_owned()),
        instructor: Some("Prof. Euler".to_owned()),
        stack_uid: Some(Uid(202)),
    }
}

#[test]
fn valid_timetable_entry_passes_validation() {
    let e = entry(0, 540, 90);
    assert!(validate_timetable_entry(&e).is_ok());
}

#[test]
fn empty_title_is_rejected() {
    let mut e = entry(0, 540, 90);
    e.title = "   ".to_owned();
    assert!(validate_timetable_entry(&e).is_err());
}

#[test]
fn invalid_weekday_is_rejected() {
    let e = entry(7, 540, 90);
    assert!(validate_timetable_entry(&e).is_err());
}

#[test]
fn invalid_time_is_rejected() {
    let e1 = entry(0, 1440, 90);
    assert!(validate_timetable_entry(&e1).is_err());

    let e2 = entry(0, 540, 0);
    assert!(validate_timetable_entry(&e2).is_err());
}

#[test]
fn converts_to_session_with_weekly_recurrence() {
    let e = entry(1, 600, 90); // Tuesday
    let term_start = Date {
        year: 2026,
        month: 11,
        day: 2,
    };
    let s = timetable_to_session(&e, term_start);
    assert_eq!(s.id, e.id);
    assert_eq!(s.title, "Calculus I");
    assert_eq!(s.duration_min, 90);
    assert_eq!(
        s.repeat,
        studyfrog_calendar::session::Repeat::Weekly {
            every: 1,
            days: 1 << 1
        }
    );
    assert!(s.notes.contains("Room: Hall B"));
    assert!(s.notes.contains("Instructor: Prof. Euler"));
}

#[test]
fn timetable_occurrences_expands_across_date_range() {
    let e1 = entry(0, 540, 90); // Monday
    let mut e2 = entry(2, 600, 60); // Wednesday
    e2.id = Uid(102);

    // 2026-11-02 is Monday, 2026-11-15 is Sunday (2 weeks)
    let from = Date {
        year: 2026,
        month: 11,
        day: 2,
    };
    let to = Date {
        year: 2026,
        month: 11,
        day: 15,
    };
    let occurrences = timetable_occurrences(&[e1, e2], from, to);

    // 2 Mondays and 2 Wednesdays = 4 occurrences
    assert_eq!(occurrences.len(), 4);
    assert_eq!(
        occurrences[0].date,
        Date {
            year: 2026,
            month: 11,
            day: 2
        }
    );
    assert_eq!(occurrences[0].start_minute, 540);
    assert_eq!(
        occurrences[1].date,
        Date {
            year: 2026,
            month: 11,
            day: 4
        }
    );
    assert_eq!(occurrences[1].start_minute, 600);
}
