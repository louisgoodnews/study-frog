//! The user's local time zone. `chrono` is used here and nowhere else: the domain crates take a UTC
//! offset as a parameter. Daylight saving is handled by asking for the offset at each instant.

use chrono::{DateTime, Local, Offset, TimeZone};
use studyfrog_core::civil::{Date, LocalTime, add_days, local_to_utc, utc_to_local};
use studyfrog_core::time::UnixMs;

use crate::day::DayWindow;

/// The local UTC offset in minutes (east positive) at an instant, from the system time zone.
pub fn local_offset_minutes(at: UnixMs) -> i32 {
    DateTime::from_timestamp_millis(at.0).map_or(0, |utc| {
        Local
            .offset_from_utc_datetime(&utc.naive_utc())
            .fix()
            .local_minus_utc()
            / 60
    })
}

/// The instant a local day begins, refining the offset twice so a day starting just after a clock
/// change is placed correctly. (A midnight that does not exist resolves to the first instant after
/// the gap; an ambiguous one to the earlier instant.)
fn local_midnight(date: Date, offset: &impl Fn(UnixMs) -> i32) -> UnixMs {
    let t = LocalTime { date, minute: 0 };
    let guess = local_to_utc(t, offset(local_to_utc(t, 0)));
    local_to_utc(t, offset(guess))
}

/// The local calendar day containing `now`, as a half-open window of UTC instants.
pub fn day_window_at(now: UnixMs, offset: impl Fn(UnixMs) -> i32) -> DayWindow {
    let today = utc_to_local(now, offset(now)).date;
    DayWindow {
        start: local_midnight(today, &offset),
        end: local_midnight(add_days(today, 1), &offset),
    }
}

/// The local day containing `now` in the system time zone.
pub fn local_day_window(now: UnixMs) -> DayWindow {
    day_window_at(now, local_offset_minutes)
}

/// The instant a local calendar day begins, in the system time zone; `day` counts from 1970-01-01.
pub fn local_day_start(day: i64) -> UnixMs {
    local_midnight(
        studyfrog_core::civil::date_from_days(day),
        &local_offset_minutes,
    )
}

/// The local calendar day (counted from 1970-01-01) an instant falls on, in the system time zone.
pub fn local_day_of(at: UnixMs) -> i64 {
    studyfrog_core::civil::date_to_days(utc_to_local(at, local_offset_minutes(at)).date)
}
