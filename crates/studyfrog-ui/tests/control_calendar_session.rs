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
