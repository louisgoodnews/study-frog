#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The diary commands of the control protocol: words, class, and the answers of the entries by day,
//! an entry opened in the editor, and a new entry (STORY-0124, EPIC-064).

mod common;
mod control_common;

use control_common::*;
use studyfrog_core::civil::parse_date;
use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_cmd::diary::{DiaryCmd, DiaryNew};
use studyfrog_ui::control_cmd::{Class, class_of};
use studyfrog_ui::control_parse::parse_request;

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

fn day(text: &str) -> studyfrog_core::civil::Date {
    parse_date(text).unwrap()
}

#[test]
fn diary_words_parse() {
    assert_eq!(
        cmd("diary"),
        Command::Diary(DiaryCmd::List {
            from: None,
            to: None
        })
    );
    assert_eq!(
        cmd("diary list 2027-03-01"),
        Command::Diary(DiaryCmd::List {
            from: Some(day("2027-03-01")),
            to: None
        })
    );
    assert_eq!(
        cmd("diary list 2027-03-01 2027-03-31"),
        Command::Diary(DiaryCmd::List {
            from: Some(day("2027-03-01")),
            to: Some(day("2027-03-31")),
        })
    );
    assert_eq!(
        cmd(
            "diary add 2027-03-01 \"Study log\" --text \"Went well\" --minutes 45 --tags focus --stacks french"
        ),
        Command::Diary(DiaryCmd::Add(DiaryNew {
            date: day("2027-03-01"),
            title: "Study log".into(),
            text: "Went well".into(),
            minutes: Some(45),
            tags: "focus".into(),
            stacks: "french".into(),
        }))
    );
    assert!(parse_request("diary open nope").is_err());
    assert!(parse_request("diary add 2027-03-01").is_err());
    assert!(parse_request("diary list soon").is_err());
    assert!(parse_request("diary list 2027-03-01 2027-03-02 2027-03-03").is_err());
}

#[test]
fn listing_and_opening_are_read_and_adding_is_write() {
    assert_eq!(class_of(&cmd("diary list 2027-03-01")), Class::Read);
    assert_eq!(
        class_of(&cmd("diary open 0000000000000000000000000A")),
        Class::Read
    );
    assert_eq!(class_of(&cmd("diary add 2027-03-01 Entry")), Class::Write);
}

#[test]
fn an_entry_is_written_listed_by_its_day_and_opened() {
    let (vault, logs) = (sample_vault("ctl-diary"), temp_dir("ctl-diary-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());

    let made = ask(
        &mut e,
        "diary add 2027-03-01 \"Control entry\" --text \"Body from the protocol\" --minutes 45 --tags focus",
    );
    assert_eq!(made["ok"], true, "{made}");
    assert_eq!(made["result"]["title"], "Control entry", "{made}");
    assert_eq!(made["result"]["date"], "2027-03-01", "{made}");
    assert_eq!(made["result"]["study_minutes"], 45, "{made}");
    let uid = made["result"]["uid"].as_str().unwrap().to_owned();

    let on_day = ask(&mut e, "diary list 2027-03-01");
    let titles: Vec<&str> = on_day["result"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|x| x["title"].as_str())
        .collect();
    assert!(titles.contains(&"Control entry"), "{on_day}");

    let other_day = ask(&mut e, "diary list 2027-03-02");
    assert_eq!(other_day["result"]["count"], 0, "{other_day}");

    let opened = ask(&mut e, &format!("diary open {uid}"));
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["result"]["editing"], true, "{opened}");
    assert_eq!(
        opened["result"]["content"], "Body from the protocol",
        "{opened}"
    );
}

#[test]
fn an_unknown_entry_is_an_error() {
    let (vault, logs) = (
        sample_vault("ctl-diary-none"),
        temp_dir("ctl-diary-none-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "diary open 0000000000000000000000000B");
    assert_eq!(r["ok"], false, "{r}");
    assert!(r["error"].as_str().unwrap().contains("no diary entry"));
}
