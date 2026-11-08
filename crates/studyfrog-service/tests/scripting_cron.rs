#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Cron schedules and the ticker (STORY-0132): parsed in-house, matched against local time, each
//! minute once, with an injected clock and no sleeping.

use studyfrog_core::civil::{LocalTime, date_new, local_to_utc};
use studyfrog_service::scripting::config::parse_scripts_toml;
use studyfrog_service::scripting::cron::{cron_matches, parse_cron};
use studyfrog_service::scripting::due::{Ticker, jobs_for_event, minutes_to_check, tick};

fn at(y: i32, mo: u8, d: u8, h: u16, mi: u16) -> LocalTime {
    LocalTime {
        date: date_new(y, mo, d).unwrap(),
        minute: h * 60 + mi,
    }
}

fn fires(expr: &str, t: LocalTime) -> bool {
    cron_matches(&parse_cron(expr).unwrap(), t)
}

#[test]
fn fields_lists_ranges_steps_and_names_match() {
    // 2026-11-09 is a Monday.
    let mon_0730 = at(2026, 11, 9, 7, 30);
    assert!(fires("30 7 * * *", mon_0730));
    assert!(!fires("31 7 * * *", mon_0730));
    assert!(fires("*/15 * * * *", mon_0730));
    assert!(!fires("*/20 * * * *", mon_0730));
    assert!(fires("0,30 6-8 * * mon-fri", mon_0730));
    assert!(!fires("30 7 * * sat,sun", mon_0730));
    assert!(fires("30 7 * nov 1", mon_0730));
    assert!(fires("30 7 9 * *", mon_0730));
    assert!(fires("5/25 7 * * *", mon_0730), "5, 30, 55");
    assert!(
        fires("30 7 * * 7", at(2026, 11, 8, 7, 30)),
        "7 is Sunday too"
    );
    assert!(fires("30 7 * * 0", at(2026, 11, 8, 7, 30)));
}

#[test]
fn day_of_month_or_weekday_when_both_are_restricted() {
    let expr = "0 9 1 * fri"; // the 1st, or any Friday
    assert!(fires(expr, at(2026, 12, 1, 9, 0)), "a Tuesday the 1st");
    assert!(fires(expr, at(2026, 11, 13, 9, 0)), "a Friday");
    assert!(!fires(expr, at(2026, 11, 12, 9, 0)));
}

#[test]
fn shorthands_and_bad_schedules() {
    assert!(fires("@daily", at(2026, 3, 4, 0, 0)));
    assert!(!fires("@daily", at(2026, 3, 4, 0, 1)));
    assert!(fires("@hourly", at(2026, 3, 4, 13, 0)));
    assert!(fires("@weekly", at(2026, 11, 8, 0, 0)));
    assert!(fires("@monthly", at(2026, 11, 1, 0, 0)));
    assert!(fires("@yearly", at(2027, 1, 1, 0, 0)));
    for (bad, says) in [
        ("* * * *", "five fields"),
        ("60 * * * *", "minute"),
        ("* 24 * * *", "hour"),
        ("* * 0 * *", "day"),
        ("* * * 13 *", "month"),
        ("* * * * funday", "weekday"),
        ("*/0 * * * *", "step of 0"),
        ("5-1 * * * *", "outside"),
    ] {
        let e = parse_cron(bad).unwrap_err();
        assert!(e.contains(says), "{bad}: {e}");
    }
}

const CONFIG: &str = r#"
[[job]]
name = "morning"
script = "morning.py"
schedule = "30 7 * * *"

[[job]]
name = "quarter"
script = "q.sh"
schedule = "*/15 * * * *"

[[job]]
name = "off"
script = "off.py"
schedule = "* * * * *"
enabled = false

[[job]]
name = "after-run"
script = "after.py"
on = ["rehearsal_finished"]
"#;

fn ms(t: LocalTime, offset: i32) -> i64 {
    local_to_utc(t, offset).0
}

#[test]
fn each_due_minute_starts_its_jobs_once() {
    let cfg = parse_scripts_toml(CONFIG, &["rehearsal_finished"]);
    assert!(cfg.errors.is_empty(), "{:?}", cfg.errors);
    let offset = 60; // UTC+1
    let mut t = Ticker::default();
    let names = tick(
        &mut t,
        &cfg,
        ms(at(2026, 11, 9, 7, 30), offset) + 5_000,
        offset,
    );
    assert_eq!(names, ["morning", "quarter"]);
    let again = tick(
        &mut t,
        &cfg,
        ms(at(2026, 11, 9, 7, 30), offset) + 50_000,
        offset,
    );
    assert!(again.is_empty(), "the same minute does not fire twice");
    let next = tick(&mut t, &cfg, ms(at(2026, 11, 9, 7, 31), offset), offset);
    assert!(next.is_empty());
    let gap = tick(&mut t, &cfg, ms(at(2026, 11, 9, 7, 50), offset), offset);
    assert_eq!(gap, ["quarter"], "7:45 was missed and is caught up once");
    let back = tick(&mut t, &cfg, ms(at(2026, 11, 9, 7, 0), offset), offset);
    assert!(back.is_empty(), "a clock that went back fires nothing");
}

#[test]
fn a_long_gap_is_caught_up_for_one_hour_only() {
    let t = Ticker {
        last_minute: Some(1000),
    };
    let r = minutes_to_check(&t, 5000 * 60_000);
    assert_eq!((*r.start(), *r.end()), (4941, 5000));
    assert_eq!(minutes_to_check(&Ticker::default(), 61_000), 1..=1);
}

#[test]
fn events_start_the_enabled_jobs_bound_to_them() {
    let cfg = parse_scripts_toml(CONFIG, &["rehearsal_finished", "item_created"]);
    let names: Vec<&str> = jobs_for_event(&cfg, "rehearsal_finished")
        .iter()
        .map(|j| j.name.as_str())
        .collect();
    assert_eq!(names, ["after-run"]);
    assert!(jobs_for_event(&cfg, "item_created").is_empty());
}
