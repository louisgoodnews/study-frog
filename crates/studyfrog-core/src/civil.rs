//! Civil (calendar) dates and local time, as plain data and pure functions. No time zone database:
//! callers supply the UTC offset in minutes, so daylight saving is the edge's concern.
//! Algorithms are Hinnant's `days_from_civil` / `civil_from_days` (proleptic Gregorian).

use crate::time::UnixMs;

const MS_PER_MINUTE: i64 = 60_000;
const MS_PER_DAY: i64 = 86_400_000;

/// A calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// Year (astronomical: year 1 BC is 0).
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u8,
    /// Day of the month, 1 to 31.
    pub day: u8,
}

/// Day of the week, Monday first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    /// Monday.
    Mon,
    /// Tuesday.
    Tue,
    /// Wednesday.
    Wed,
    /// Thursday.
    Thu,
    /// Friday.
    Fri,
    /// Saturday.
    Sat,
    /// Sunday.
    Sun,
}

/// All weekdays, Monday first.
pub const WEEKDAYS: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

/// Position of a weekday, Monday = 0.
pub fn weekday_index(w: Weekday) -> u8 {
    w as u8
}

/// Days since 1970-01-01 for a proleptic Gregorian date.
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverse of [`days_from_civil`]: (year, month, day).
pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Days in a month of a year.
pub fn days_in_month(year: i64, month: i64) -> i64 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// A validated date, or `None` (month 13, 30 February, ...).
pub fn date_new(year: i32, month: u8, day: u8) -> Option<Date> {
    let ok = (1..=12).contains(&month)
        && day >= 1
        && i64::from(day) <= days_in_month(i64::from(year), i64::from(month));
    ok.then_some(Date { year, month, day })
}

/// Days since 1970-01-01.
pub fn date_to_days(d: Date) -> i64 {
    days_from_civil(i64::from(d.year), i64::from(d.month), i64::from(d.day))
}

/// The date `days` days after 1970-01-01.
pub fn date_from_days(days: i64) -> Date {
    let (y, m, d) = civil_from_days(days);
    Date {
        year: y as i32,
        month: m as u8,
        day: d as u8,
    }
}

/// The date `n` days after (or before, when negative) `d`.
pub fn add_days(d: Date, n: i64) -> Date {
    date_from_days(date_to_days(d) + n)
}

/// Days from `a` to `b` (negative when `b` is earlier).
pub fn days_between(a: Date, b: Date) -> i64 {
    date_to_days(b) - date_to_days(a)
}

/// The weekday of a date.
pub fn weekday(d: Date) -> Weekday {
    // 1970-01-01 was a Thursday.
    WEEKDAYS[(date_to_days(d) + 3).rem_euclid(7) as usize]
}

/// `d` moved `months` months forward keeping the day of the month; `None` when that month has no
/// such day (31 January plus one month).
pub fn add_months(d: Date, months: i64) -> Option<Date> {
    let index = i64::from(d.year) * 12 + i64::from(d.month) - 1 + months;
    date_new(
        index.div_euclid(12) as i32,
        (index.rem_euclid(12) + 1) as u8,
        d.day,
    )
}

/// The 42 dates (six weeks) of a month view starting on `week_start`.
pub fn month_grid(year: i32, month: u8, week_start: Weekday) -> [Date; 42] {
    let first = Date {
        year,
        month,
        day: 1,
    };
    let back = (i64::from(weekday_index(weekday(first))) - i64::from(weekday_index(week_start)))
        .rem_euclid(7);
    let start = add_days(first, -back);
    std::array::from_fn(|i| add_days(start, i as i64))
}

/// A wall-clock minute of the day (0 to 1439) on a date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalTime {
    /// The date.
    pub date: Date,
    /// Minutes since local midnight.
    pub minute: u16,
}

/// The UTC instant of a local date-time, given the local UTC offset in minutes (east positive).
pub fn local_to_utc(t: LocalTime, offset_minutes: i32) -> UnixMs {
    let local_ms = date_to_days(t.date) * MS_PER_DAY + i64::from(t.minute) * MS_PER_MINUTE;
    UnixMs(local_ms - i64::from(offset_minutes) * MS_PER_MINUTE)
}

/// The local date-time of a UTC instant, given the local UTC offset in minutes (east positive).
pub fn utc_to_local(at: UnixMs, offset_minutes: i32) -> LocalTime {
    let local_ms = at.0 + i64::from(offset_minutes) * MS_PER_MINUTE;
    let days = local_ms.div_euclid(MS_PER_DAY);
    let minute = (local_ms.rem_euclid(MS_PER_DAY) / MS_PER_MINUTE) as u16;
    LocalTime {
        date: date_from_days(days),
        minute,
    }
}

/// `YYYY-MM-DD`.
pub fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// Parse `YYYY-MM-DD` (four-digit year); `None` for anything else or a date that does not exist.
pub fn parse_date(text: &str) -> Option<Date> {
    let b = text.as_bytes();
    let digits = |r: std::ops::Range<usize>| -> Option<u32> {
        let s = text.get(r)?;
        s.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| s.parse().ok())
            .flatten()
    };
    let ok = b.len() == 10 && b[4] == b'-' && b[7] == b'-';
    if !ok {
        return None;
    }
    date_new(
        i32::try_from(digits(0..4)?).ok()?,
        u8::try_from(digits(5..7)?).ok()?,
        u8::try_from(digits(8..10)?).ok()?,
    )
}

/// `HH:MM` for minutes since midnight.
pub fn format_minute(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

/// Parse `HH:MM` (24-hour) into minutes since midnight.
pub fn parse_minute(text: &str) -> Option<u16> {
    let (h, m) = text.trim().split_once(':')?;
    let (h, m): (u16, u16) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60 && text.trim().len() <= 5).then_some(h * 60 + m)
}
