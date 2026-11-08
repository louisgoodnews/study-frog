#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Focus blocks recorded against the calendar study session they overlap (STORY-0112).

mod recall_common;

use recall_common::*;
use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_service::focus::{FocusDone, record_focus_at};
use studyfrog_service::planning::save_session_checked;
use studyfrog_store::focus::focus_minutes_for;

/// 2026-11-08 00:00 UTC.
const MIDNIGHT: i64 = 1_794_096_000_000;
const MIN: i64 = 60_000;
const DAY: Date = Date {
    year: 2026,
    month: 11,
    day: 8,
};

fn session(id: u128, title: &str, (minute, length): (u16, u16), kind: EventKind) -> Session {
    Session {
        id: Uid(id),
        title: title.into(),
        stack: None,
        start: LocalTime { date: DAY, minute },
        duration_min: length,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind,
        reminders: Vec::new(),
    }
}

fn block(from_min: i64, minutes: u32) -> FocusDone {
    FocusDone {
        started_ms: MIDNIGHT + from_min * MIN,
        ended_ms: MIDNIGHT + (from_min + i64::from(minutes)) * MIN,
        minutes,
        stack: Some("bio".into()),
    }
}

#[test]
fn a_block_counts_towards_the_study_session_it_overlaps_most() {
    let dir = vault("focus-session", &[]);
    let mut s = open(&dir);
    for x in [
        session(20, "Statistics", (9 * 60, 60), EventKind::Study),
        session(21, "Biology", (10 * 60, 90), EventKind::Study),
        session(22, "Dentist", (9 * 60, 120), EventKind::Appointment),
    ] {
        save_session_checked(&mut s, x).unwrap();
    }
    let done = record_focus_at(&s, &block(9 * 60 + 50, 25), 0).unwrap();
    assert_eq!(done.minutes, 25);
    assert_eq!(
        done.session,
        Some((Uid(21), "Biology".to_owned(), DAY)),
        "15 minutes beat 10"
    );
    assert_eq!(focus_minutes_for(&s.conn, Uid(21), DAY).unwrap(), 25);
    let again = record_focus_at(&s, &block(10 * 60 + 30, 25), 0).unwrap();
    assert_eq!(again.today, 50);
    assert_eq!(focus_minutes_for(&s.conn, Uid(21), DAY).unwrap(), 50);
    assert_eq!(focus_minutes_for(&s.conn, Uid(20), DAY).unwrap(), 0);
}

#[test]
fn a_block_outside_every_session_is_still_recorded_and_the_offset_counts() {
    let dir = vault("focus-none", &[]);
    let mut s = open(&dir);
    save_session_checked(
        &mut s,
        session(20, "Statistics", (9 * 60, 60), EventKind::Study),
    )
    .unwrap();
    let done = record_focus_at(&s, &block(14 * 60, 50), 0).unwrap();
    assert_eq!(done.session, None);
    assert_eq!(done.today, 50);
    let shifted = record_focus_at(&s, &block(8 * 60, 30), 60).unwrap();
    assert_eq!(
        shifted.session.map(|x| x.0),
        Some(Uid(20)),
        "08:00 UTC is 09:00 at +01:00"
    );
}
