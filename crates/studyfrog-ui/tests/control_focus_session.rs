#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The focus and tooltip answers of the control protocol (STORY-0192): `search --focus` answers whether the
//! request is still pending, `help tooltip NAME` gives the explanation, and the state names the focused field.

mod common;
mod control_common;

use control_common::*;
use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_parse::parse_request;

#[test]
fn search_focus_and_help_tooltip_parse_to_their_commands() {
    let cmd = |l: &str| parse_request(l).unwrap().cmd;
    assert_eq!(cmd("search --focus"), Command::Search(String::new(), true));
    assert_eq!(
        cmd("search --focus le chat"),
        Command::Search("le chat".into(), true)
    );
    assert_eq!(
        cmd("search le chat"),
        Command::Search("le chat".into(), false)
    );
    assert_eq!(cmd("help"), Command::Help(None));
    assert_eq!(
        cmd("help tooltip Back (Alt+Left)"),
        Command::Help(Some("Back (Alt+Left)".into()))
    );
    assert!(parse_request("help tooltip").is_err());
    assert!(parse_request("help nope").is_err());
}

#[test]
fn a_tooltip_is_answered_by_name_and_the_focus_request_is_pending_without_a_frame() {
    let (vault, logs) = (sample_vault("ctl-focus"), temp_dir("ctl-focus-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());

    let r = ask(&mut e, "help tooltip Back (Alt+Left)");
    assert_eq!(r["ok"], true, "{r}");
    assert!(r["result"]["text"].as_str().unwrap().contains("Alt+Left"));

    let r = ask(&mut e, "help tooltip Nope");
    assert_eq!(r["ok"], false, "{r}");
    assert!(r["error"].as_str().unwrap().contains("names are"));

    let r = ask(&mut e, "search --focus");
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["result"]["focus"]["pending"], true, "{r}");
    assert!(r["result"]["focus"]["focused"].is_null());

    let state = ask(&mut e, "state");
    assert!(state["result"]["focused_field"].is_null(), "{state}");
    assert_eq!(state["result"]["search_focus_pending"], true, "{state}");
}
