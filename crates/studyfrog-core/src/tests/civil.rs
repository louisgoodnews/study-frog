use proptest::prelude::*;

use crate::civil::{
    Date, LocalTime, Weekday, add_days, add_months, date_from_days, date_new, date_to_days,
    days_between, days_in_month, local_to_utc, month_grid, utc_to_local, weekday,
};
use crate::time::UnixMs;

fn d(y: i32, m: u8, day: u8) -> Date {
    date_new(y, m, day).unwrap()
}

#[test]
fn known_dates_and_weekdays() {
    assert_eq!(date_to_days(d(1970, 1, 1)), 0);
    assert_eq!(weekday(d(1970, 1, 1)), Weekday::Thu);
    assert_eq!(weekday(d(2026, 10, 30)), Weekday::Fri);
    assert_eq!(weekday(d(2000, 2, 29)), Weekday::Tue);
    assert_eq!(days_between(d(2026, 1, 1), d(2027, 1, 1)), 365);
    assert_eq!(days_between(d(2024, 1, 1), d(2025, 1, 1)), 366);
}

#[test]
fn only_real_dates_are_accepted() {
    for (y, m, day) in [(2024, 2, 29), (2000, 2, 29), (2026, 12, 31)] {
        assert!(date_new(y, m, day).is_some());
    }
    for (y, m, day) in [
        (2023, 2, 29),
        (1900, 2, 29),
        (2026, 13, 1),
        (2026, 4, 31),
        (2026, 1, 0),
        (2026, 0, 5),
    ] {
        assert!(date_new(y, m, day).is_none(), "{y}-{m}-{day}");
    }
    assert_eq!(
        (
            days_in_month(2024, 2),
            days_in_month(2100, 2),
            days_in_month(2026, 11)
        ),
        (29, 28, 30)
    );
}

#[test]
fn months_keep_the_day_or_give_up() {
    assert_eq!(add_months(d(2026, 1, 15), 13), Some(d(2027, 2, 15)));
    assert_eq!(add_months(d(2026, 1, 31), 1), None, "no 31 February");
    assert_eq!(add_months(d(2026, 1, 31), 2), Some(d(2026, 3, 31)));
    assert_eq!(add_months(d(2026, 3, 10), -3), Some(d(2025, 12, 10)));
}

#[test]
fn the_month_grid_starts_on_the_chosen_weekday_and_covers_the_month() {
    let grid = month_grid(2026, 10, Weekday::Mon);
    assert_eq!(grid[0], d(2026, 9, 28));
    assert_eq!(grid[41], d(2026, 11, 8));
    assert!((1..=31).all(|day| grid.contains(&d(2026, 10, day))));
    assert_eq!(month_grid(2026, 10, Weekday::Sun)[0], d(2026, 9, 27));
    assert_eq!(
        month_grid(2026, 2, Weekday::Mon)[0],
        d(2026, 1, 26),
        "a month starting on Sunday"
    );
}

#[test]
fn local_time_converts_with_an_offset_and_back() {
    let t = LocalTime {
        date: d(2026, 10, 30),
        minute: 9 * 60 + 30,
    };
    let utc = local_to_utc(t, 120); // UTC+2
    assert_eq!(utc, UnixMs(1_793_345_400_000));
    assert_eq!(utc_to_local(utc, 120), t);
    let midnight = utc_to_local(UnixMs(0), -300);
    assert_eq!(
        (midnight.date, midnight.minute),
        (d(1969, 12, 31), 19 * 60),
        "west of Greenwich is the evening before"
    );
}

proptest! {
    #[test]
    fn days_and_dates_round_trip(n in -800_000_i64..800_000) {
        let date = date_from_days(n);
        prop_assert_eq!(date_to_days(date), n);
        prop_assert!(date_new(date.year, date.month, date.day).is_some());
        prop_assert_eq!(add_days(date, 1), date_from_days(n + 1));
    }

    #[test]
    fn weekdays_advance_by_one_per_day(n in -100_000_i64..100_000) {
        let a = weekday(date_from_days(n)) as u8;
        let b = weekday(date_from_days(n + 1)) as u8;
        prop_assert_eq!((a + 1) % 7, b);
    }

    #[test]
    fn local_conversion_round_trips_for_any_offset(ms in -4_000_000_000_000_i64..8_000_000_000_000, off in -720_i32..=840) {
        let local = utc_to_local(UnixMs(ms), off);
        let back = local_to_utc(local, off);
        // Local time has minute resolution, so the round trip drops sub-minute remainders.
        prop_assert_eq!(back.0, ms - ms.rem_euclid(60_000));
    }
}

#[test]
fn dates_and_times_parse_and_format_strictly() {
    use crate::civil::{format_date, format_minute, parse_date, parse_minute};
    assert_eq!(format_date(d(2026, 3, 9)), "2026-03-09");
    assert_eq!(parse_date("2026-03-09"), Some(d(2026, 3, 9)));
    for bad in [
        "2026-3-9",
        "2026-02-30",
        "26-03-09",
        "2026/03/09",
        "2026-03-09x",
        "",
        "2026-13-01",
        "+026-03-09",
    ] {
        assert_eq!(parse_date(bad), None, "{bad:?}");
    }
    assert_eq!(
        (
            format_minute(0),
            format_minute(9 * 60 + 5),
            format_minute(1439)
        ),
        ("00:00".into(), "09:05".into(), "23:59".into())
    );
    assert_eq!(
        (
            parse_minute("9:05"),
            parse_minute(" 23:59 "),
            parse_minute("24:00"),
            parse_minute("12:60"),
            parse_minute("noon")
        ),
        (Some(545), Some(1439), None, None, None)
    );
}

proptest! {
    #[test]
    fn date_text_round_trips(n in -300_000_i64..300_000) {
        use crate::civil::{format_date, parse_date};
        let date = date_from_days(n);
        prop_assume!((0..=9999).contains(&date.year));
        prop_assert_eq!(parse_date(&format_date(date)), Some(date));
    }
}
