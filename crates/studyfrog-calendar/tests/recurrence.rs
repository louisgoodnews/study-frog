#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Recurrence expansion against a naive day-by-day oracle, plus the cases people actually hit.

use proptest::prelude::*;
use studyfrog_calendar::recurrence::occurrences;
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{
    Date, LocalTime, add_days, date_new, days_between, weekday, weekday_index,
};
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

/// Does the rule put an occurrence on `day`? (ignoring ends and cancellations)
fn matches_rule(s: &Session, day: Date) -> bool {
    let start = s.start.date;
    if day < start {
        return false;
    }
    match s.repeat {
        Repeat::Never => day == start,
        Repeat::Daily { every } => days_between(start, day) % i64::from(every.max(1)) == 0,
        Repeat::Weekly { every, days } => {
            let mask = if days == 0 {
                1 << weekday_index(weekday(start))
            } else {
                days
            };
            let week = |x: Date| (days_between(d(1970, 1, 5), x)).div_euclid(7); // 1970-01-05 was a Monday
            mask & (1 << weekday_index(weekday(day))) != 0
                && (week(day) - week(start)) % i64::from(every.max(1)) == 0
        }
        Repeat::Monthly { every } => {
            let months = (i64::from(day.year) - i64::from(start.year)) * 12 + i64::from(day.month)
                - i64::from(start.month);
            day.day == start.day && months % i64::from(every.max(1)) == 0
        }
    }
}

/// Every occurrence from the start up to `to`, found by testing each day.
fn oracle(s: &Session, from: Date, to: Date) -> Vec<Date> {
    let (mut out, mut count) = (Vec::new(), 0_u32);
    let mut day = s.start.date;
    while day <= to {
        let allowed = match s.ends {
            Ends::Forever => true,
            Ends::On(last) => day <= last,
            Ends::After(n) => count < n,
        };
        if matches_rule(s, day) && allowed {
            count += 1;
            if day >= from && !s.cancelled.contains(&day) {
                out.push(day);
            }
        }
        day = add_days(day, 1);
    }
    out
}

fn repeat() -> impl Strategy<Value = Repeat> {
    prop_oneof![
        Just(Repeat::Never),
        (1_u16..5).prop_map(|every| Repeat::Daily { every }),
        (1_u16..4, 0_u8..128).prop_map(|(every, days)| Repeat::Weekly { every, days }),
        (1_u16..14).prop_map(|every| Repeat::Monthly { every }),
    ]
}

fn day() -> impl Strategy<Value = Date> {
    (2024_i32..2027, 1_u8..=12, 1_u8..=31)
        .prop_filter_map("real date", |(y, m, dd)| date_new(y, m, dd))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn expansion_matches_the_day_by_day_oracle(
        start in day(),
        repeat in repeat(),
        ends in prop_oneof![Just(Ends::Forever), day().prop_map(Ends::On), (0_u32..30).prop_map(Ends::After)],
        window in (0_i64..500, 0_i64..120),
        cancelled in prop::collection::vec(day(), 0..4),
    ) {
        let mut s = session(start, repeat, ends);
        s.cancelled = cancelled;
        let from = add_days(start, window.0 - 60);
        let to = add_days(from, window.1);
        prop_assert_eq!(dates(&s, from, to), oracle(&s, from, to));
    }

    #[test]
    fn results_are_sorted_unique_inside_the_range(start in day(), repeat in repeat(), span in 0_i64..400) {
        let s = session(start, repeat, Ends::Forever);
        let (from, to) = (start, add_days(start, span));
        let got = dates(&s, from, to);
        prop_assert!(got.windows(2).all(|w| w[0] < w[1]));
        prop_assert!(got.iter().all(|x| *x >= from && *x <= to));
    }
}
