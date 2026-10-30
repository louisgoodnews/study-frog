//! Expanding a session into its occurrences within a date range. Pure and bounded.

use studyfrog_core::civil::{
    Date, add_days, add_months, date_from_days, date_to_days, weekday, weekday_index,
};

use crate::session::{Ends, Occurrence, Repeat, Session};

/// Most occurrences an `After` limit may ask for.
pub const MAX_AFTER: u32 = 10_000;
/// Hard bound on the periods examined in one call.
const MAX_PERIODS: i64 = 40_000;

fn every(n: u16) -> i64 {
    i64::from(n.max(1))
}

/// Monday of the week containing `d`, as a day number.
fn week_start_days(d: Date) -> i64 {
    date_to_days(d) - i64::from(weekday_index(weekday(d)))
}

/// The dates of period `k` (`k >= 0`) of the rule, in order. A period is a day, a week or a month.
fn period_dates(s: &Session, k: i64) -> Vec<Date> {
    let start = s.start.date;
    match s.repeat {
        Repeat::Never => {
            if k == 0 {
                vec![start]
            } else {
                Vec::new()
            }
        }
        Repeat::Daily { every: n } => vec![add_days(start, k * every(n))],
        Repeat::Weekly { every: n, days } => {
            let monday = week_start_days(start) + k * 7 * every(n);
            let mask = if days == 0 {
                1 << weekday_index(weekday(start))
            } else {
                days
            };
            (0..7)
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| date_from_days(monday + i))
                .collect()
        }
        Repeat::Monthly { every: n } => add_months(start, k * every(n)).into_iter().collect(),
    }
}

/// The period to begin scanning at for a range starting at `from` (one early, for safety).
fn first_period(s: &Session, from: Date) -> i64 {
    let start = s.start.date;
    let gap = match s.repeat {
        Repeat::Never => return 0,
        Repeat::Daily { every: n } => (date_to_days(from) - date_to_days(start)) / every(n),
        Repeat::Weekly { every: n, .. } => {
            (date_to_days(from) - week_start_days(start)) / (7 * every(n))
        }
        Repeat::Monthly { every: n } => {
            let months = (i64::from(from.year) - i64::from(start.year)) * 12
                + i64::from(from.month)
                - i64::from(start.month);
            months / every(n)
        }
    };
    (gap - 1).max(0)
}

fn limit(s: &Session) -> Option<Date> {
    match s.ends {
        Ends::On(d) => Some(d),
        Ends::Forever | Ends::After(_) => None,
    }
}

/// The occurrences of `s` on dates from `from` to `to`, both included, in date order.
pub fn occurrences(s: &Session, from: Date, to: Date) -> Vec<Occurrence> {
    let mut out = Vec::new();
    if to < from {
        return out;
    }
    let after = match s.ends {
        Ends::After(n) => Some(u64::from(n.min(MAX_AFTER))),
        _ => None,
    };
    // An `After` limit counts from the very first occurrence, so those scan from period 0.
    let start = if after.is_some() {
        0
    } else {
        first_period(s, from)
    };
    let mut seen: u64 = if after.is_some() { 0 } else { u64::MAX };
    for k in (start..).take(MAX_PERIODS as usize) {
        let dates = period_dates(s, k);
        if dates.first().is_some_and(|d| *d > to)
            || (dates.is_empty() && matches!(s.repeat, Repeat::Never))
        {
            break;
        }
        for d in dates.into_iter().filter(|d| *d >= s.start.date) {
            if after.is_some_and(|n| seen >= n) || limit(s).is_some_and(|l| d > l) {
                return out;
            }
            seen = seen.wrapping_add(1);
            if d >= from && d <= to && !s.cancelled.contains(&d) {
                out.push(Occurrence {
                    session: s.id,
                    date: d,
                    start_minute: s.start.minute,
                    duration_min: s.duration_min,
                });
            }
        }
        if matches!(s.repeat, Repeat::Never) {
            break;
        }
    }
    out
}

/// The occurrences of all sessions in the range, ordered by date, start time and session id.
pub fn occurrences_all(sessions: &[Session], from: Date, to: Date) -> Vec<Occurrence> {
    let mut all: Vec<Occurrence> = sessions
        .iter()
        .flat_map(|s| occurrences(s, from, to))
        .collect();
    all.sort_by_key(|o| (o.date, o.start_minute, o.session));
    all
}
