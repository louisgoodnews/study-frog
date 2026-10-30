#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Recurrence: the cases people actually hit.

use studyfrog_calendar::recurrence::{occurrences, occurrences_all};
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{Date, LocalTime, date_new};
use studyfrog_core::uid::Uid;

fn d(y: i32, m: u8, day: u8) -> Date {
    date_new(y, m, day).unwrap()
}

fn session(start: Date, repeat: Repeat, ends: Ends) -> Session {
    Session {
        id: Uid(1),
        title: "S".into(),
        stack: None,
        start: LocalTime {
            date: start,
            minute: 9 * 60,
        },
        duration_min: 30,
        repeat,
        ends,
        cancelled: Vec::new(),
        notes: String::new(),
    }
}

fn dates(s: &Session, from: Date, to: Date) -> Vec<Date> {
    occurrences(s, from, to)
        .into_iter()
        .map(|o| o.date)
        .collect()
}

#[test]
fn a_session_that_never_repeats_happens_once() {
    let s = session(d(2026, 11, 3), Repeat::Never, Ends::Forever);
    assert_eq!(dates(&s, d(2026, 1, 1), d(2027, 1, 1)), [d(2026, 11, 3)]);
    assert!(dates(&s, d(2026, 11, 4), d(2027, 1, 1)).is_empty());
    assert!(
        dates(&s, d(2026, 12, 1), d(2026, 11, 1)).is_empty(),
        "an empty range"
    );
}

#[test]
fn weekly_on_chosen_weekdays_every_other_week() {
    // Tuesday and Thursday (bits 1 and 3), every 2 weeks, from Tuesday 3 Nov 2026.
    let s = session(
        d(2026, 11, 3),
        Repeat::Weekly {
            every: 2,
            days: 0b0001010,
        },
        Ends::Forever,
    );
    assert_eq!(
        dates(&s, d(2026, 11, 1), d(2026, 12, 10)),
        [
            d(2026, 11, 3),
            d(2026, 11, 5),
            d(2026, 11, 17),
            d(2026, 11, 19),
            d(2026, 12, 1),
            d(2026, 12, 3)
        ]
    );
    let none = session(
        d(2026, 11, 4),
        Repeat::Weekly { every: 1, days: 0 },
        Ends::After(3),
    );
    assert_eq!(
        dates(&none, d(2026, 11, 1), d(2027, 1, 1)),
        [d(2026, 11, 4), d(2026, 11, 11), d(2026, 11, 18)],
        "an empty mask means the start weekday"
    );
}

#[test]
fn monthly_on_the_31st_skips_short_months() {
    let s = session(d(2026, 1, 31), Repeat::Monthly { every: 1 }, Ends::Forever);
    assert_eq!(
        dates(&s, d(2026, 1, 1), d(2026, 7, 31)),
        [
            d(2026, 1, 31),
            d(2026, 3, 31),
            d(2026, 5, 31),
            d(2026, 7, 31)
        ]
    );
}

#[test]
fn limits_and_cancellations() {
    let mut s = session(d(2026, 11, 1), Repeat::Daily { every: 1 }, Ends::After(5));
    s.cancelled = vec![d(2026, 11, 2)];
    assert_eq!(
        dates(&s, d(2026, 11, 1), d(2026, 12, 31)),
        [
            d(2026, 11, 1),
            d(2026, 11, 3),
            d(2026, 11, 4),
            d(2026, 11, 5)
        ],
        "a cancelled day still counts towards the five"
    );
    let until = session(
        d(2026, 11, 1),
        Repeat::Daily { every: 3 },
        Ends::On(d(2026, 11, 10)),
    );
    assert_eq!(
        dates(&until, d(2026, 11, 1), d(2027, 1, 1)),
        [
            d(2026, 11, 1),
            d(2026, 11, 4),
            d(2026, 11, 7),
            d(2026, 11, 10)
        ]
    );
}

#[test]
fn a_far_future_window_is_cheap_and_correct() {
    let s = session(
        d(2024, 1, 1),
        Repeat::Weekly {
            every: 1,
            days: 0b0000001,
        },
        Ends::Forever,
    );
    let got = dates(&s, d(2030, 1, 1), d(2030, 1, 31));
    assert_eq!(
        got,
        [
            d(2030, 1, 7),
            d(2030, 1, 14),
            d(2030, 1, 21),
            d(2030, 1, 28)
        ]
    );
}

#[test]
fn several_sessions_interleave_by_date_and_time() {
    let mut a = session(d(2026, 11, 2), Repeat::Daily { every: 1 }, Ends::After(2));
    let mut b = session(d(2026, 11, 2), Repeat::Never, Ends::Forever);
    (a.id, b.id) = (Uid(2), Uid(1));
    b.start.minute = 8 * 60;
    let all = occurrences_all(&[a, b], d(2026, 11, 1), d(2026, 11, 30));
    let order: Vec<_> = all
        .iter()
        .map(|o| (o.date.day, o.start_minute, o.session.0))
        .collect();
    assert_eq!(order, [(2, 480, 1), (2, 540, 2), (3, 540, 2)]);
}
