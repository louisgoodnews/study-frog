#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calendar commands of the control protocol (STORY-0194): their parsing, and the answers of the
//! homework and of `reminders due`, which is read without marking anything shown.

mod common;
mod control_common;

use control_common::*;
use studyfrog_core::civil::parse_date;
use studyfrog_ui::calendar_mode::CalendarViewMode;
use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_cmd::calendar::CalendarCmd;
use studyfrog_ui::control_parse::parse_request;

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

#[test]
fn calendar_and_reminders_words_parse() {
    let day = parse_date("2027-03-01").unwrap();
    assert_eq!(
        cmd("calendar view week"),
        Command::Calendar(CalendarCmd::View(CalendarViewMode::Week))
    );
    assert_eq!(
        cmd("calendar view timetable"),
        Command::Calendar(CalendarCmd::View(CalendarViewMode::Timetable))
    );
    assert_eq!(
        cmd("calendar period 2027-03-01"),
        Command::Calendar(CalendarCmd::Period(day))
    );
    assert_eq!(
        cmd("calendar homework 2027-03-01"),
        Command::Calendar(CalendarCmd::Homework(day))
    );
    assert_eq!(
        cmd("reminders due"),
        Command::Calendar(CalendarCmd::RemindersDue)
    );
    assert!(parse_request("calendar view year").is_err());
    assert!(
        parse_request("calendar period soon")
            .unwrap_err()
            .1
            .contains("date")
    );
    assert!(parse_request("calendar").is_err());
    assert!(parse_request("reminders later").is_err());
}

#[test]
fn reminders_due_is_read_again_and_nothing_is_marked_shown() {
    let (vault, logs) = (sample_vault("ctl-cal"), temp_dir("ctl-cal-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());

    let first = ask(&mut e, "reminders due");
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(first["result"]["loaded"], true, "{first}");
    assert!(first["result"]["sessions"].is_array());
    assert!(first["result"]["homework"].is_array());

    let second = ask(&mut e, "reminders due");
    assert_eq!(second["result"]["sessions"], first["result"]["sessions"]);
    assert_eq!(second["result"]["homework"], first["result"]["homework"]);
}

#[test]
fn the_calendar_homework_answer_names_the_day() {
    let (vault, logs) = (sample_vault("ctl-cal-hw"), temp_dir("ctl-cal-hw-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "calendar homework 2027-03-01");
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["result"]["day"], "2027-03-01", "{r}");
}

#[test]
fn calendar_session_add_creates_the_session_and_reminders_due_reports_it() {
    use studyfrog_core::civil::{format_date, format_minute, utc_to_local};
    use studyfrog_core::time::UnixMs;
    use studyfrog_service::timezone::local_offset_minutes;

    let (vault, logs) = (sample_vault("ctl-cal-add"), temp_dir("ctl-cal-add-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());

    // A start five minutes after the clock, on the local clock (the reminder lead is 15 minutes).
    let soon = e.model.state.wall_ms + 5 * 60_000;
    let local = utc_to_local(UnixMs(soon), local_offset_minutes(UnixMs(soon)));
    let start = format!(
        "{} {}",
        format_date(local.date),
        format_minute(local.minute)
    );

    let line = format!("calendar session add \"Soon session\" --start \"{start}\" --minutes 30");
    let made = ask(&mut e, &line);
    assert_eq!(made["ok"], true, "{made}");
    assert_eq!(made["result"]["session"]["title"], "Soon session", "{made}");
    assert_eq!(made["result"]["session"]["start"], start.as_str(), "{made}");
    assert_eq!(made["result"]["session"]["minutes"], 30, "{made}");

    let due = ask(&mut e, "reminders due");
    assert_eq!(due["ok"], true, "{due}");
    let titles: Vec<&Value> = due["result"]["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| &s["title"])
        .collect();
    assert!(titles.contains(&&json!("Soon session")), "{due}");

    // Reading does not mark it shown: the next read still lists it.
    let again = ask(&mut e, "reminders due");
    assert_eq!(
        again["result"]["sessions"], due["result"]["sessions"],
        "{again}"
    );
}

#[test]
fn calendar_session_add_words_need_a_start_and_a_length() {
    assert!(parse_request("calendar session add \"T\" --minutes 30").is_err());
    assert!(parse_request("calendar session add \"T\" --start \"2027-03-01 09:00\"").is_err());
    assert!(
        parse_request("calendar session add \"T\" --start \"2027-03-01 09:00\" --minutes 0")
            .is_err()
    );
    assert!(parse_request("calendar session edit \"T\"").is_err());
    let ok = cmd("calendar session add \"Lecture\" --start \"2027-03-01 09:30\" --minutes 90");
    assert!(
        matches!(ok, Command::Calendar(CalendarCmd::SessionAdd(_))),
        "{ok:?}"
    );
}
