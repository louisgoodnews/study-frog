#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calendar view and period answer with the selection, and `homework tab` with the tab in front
//! (EPIC-064 criterion 3).

mod common;
mod control_common;

use control_common::*;
use studyfrog_ui::calendar_mode::CalendarViewMode;
use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_cmd::calendar::CalendarCmd;
use studyfrog_ui::control_parse::parse_request;

#[test]
fn a_bare_view_or_period_is_the_read_form() {
    assert_eq!(
        parse_request("calendar view").unwrap().cmd,
        Command::Calendar(CalendarCmd::Show)
    );
    assert_eq!(
        parse_request("calendar period").unwrap().cmd,
        Command::Calendar(CalendarCmd::Show)
    );
}

#[test]
fn the_calendar_answers_the_view_and_the_period_it_shows() {
    let (vault, logs) = (sample_vault("ctl-view"), temp_dir("ctl-view-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());

    let period = ask(&mut e, "calendar period 2027-03-01");
    assert_eq!(period["ok"], true, "{period}");
    assert_eq!(period["result"]["view"], "month", "{period}");
    assert_eq!(period["result"]["month"], "2027-03", "{period}");
    assert_eq!(period["result"]["selected"], "2027-03-01", "{period}");
    assert!(period["result"]["sessions"].is_array(), "{period}");

    let view = ask(&mut e, "calendar view week");
    assert_eq!(view["result"]["view"], "week", "{view}");
    assert_eq!(view["result"]["selected"], "2027-03-01", "{view}");

    let bare = ask(&mut e, "calendar view");
    assert_eq!(bare["result"]["view"], "week", "{bare}");

    let state = ask(&mut e, "state");
    assert_eq!(state["result"]["planner"]["view"], "week", "{state}");
    assert_eq!(
        state["result"]["planner"]["selected"], "2027-03-01",
        "{state}"
    );
}

#[test]
fn every_calendar_view_word_is_taken() {
    for word in ["month", "week", "day", "timetable"] {
        let line = format!("calendar view {word}");
        assert!(
            matches!(
                parse_request(&line).unwrap().cmd,
                Command::Calendar(CalendarCmd::View(_))
            ),
            "{line}"
        );
    }
    assert_eq!(
        parse_request("calendar view timetable").unwrap().cmd,
        Command::Calendar(CalendarCmd::View(CalendarViewMode::Timetable))
    );
}

#[test]
fn homework_tab_answers_the_tab_in_front() {
    let (vault, logs) = (sample_vault("ctl-tab"), temp_dir("ctl-tab-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());

    let r = ask(&mut e, "homework tab overdue");
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["result"]["tab"], "overdue", "{r}");
    assert!(r["result"]["subject"].is_null(), "{r}");

    let all = ask(&mut e, "homework tab all");
    assert_eq!(all["result"]["tab"], "all", "{all}");
}
