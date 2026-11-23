#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The screens that had only `nav` (achievements, graph, assistant, settings, match, folders, PDF open)
//! and `stack delete`: words, class, and headless answers on a sample vault (STORY-0124, EPIC-064).

mod common;
mod control_common;

use control_common::*;
use studyfrog_ui::control_cmd::screens::{
    AchFilter, AchievementsCmd, AssistantCmd, FoldersCmd, MatchCmd, ScreenCmd, SettingsCmd,
};
use studyfrog_ui::control_cmd::{Class, Command, StackCmd, class_of};

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

fn screen(line: &str) -> ScreenCmd {
    match cmd(line) {
        Command::Screens(c) => c,
        other => panic!("{line} is not a screen command: {other:?}"),
    }
}

#[test]
fn the_screen_words_parse_to_their_commands() {
    assert_eq!(
        screen("achievements"),
        ScreenCmd::Achievements(AchievementsCmd::List)
    );
    assert_eq!(
        screen("achievements filter stack french"),
        ScreenCmd::Achievements(AchievementsCmd::Filter(AchFilter::Stack("french".into())))
    );
    assert_eq!(screen("graph"), ScreenCmd::Graph);
    assert_eq!(
        screen("assistant"),
        ScreenCmd::Assistant(AssistantCmd::State)
    );
    assert_eq!(
        screen("assistant send \"Explain recall\""),
        ScreenCmd::Assistant(AssistantCmd::Send("Explain recall".into()))
    );
    assert_eq!(screen("settings"), ScreenCmd::Settings(SettingsCmd::Read));
    assert_eq!(
        screen("settings slow-answer off"),
        ScreenCmd::Settings(SettingsCmd::SlowAnswer(false))
    );
    assert_eq!(screen("match"), ScreenCmd::Match(MatchCmd::State));
    assert_eq!(screen("match start"), ScreenCmd::Match(MatchCmd::Start));
    assert_eq!(screen("folders"), ScreenCmd::Folders(FoldersCmd::List));
    assert_eq!(
        screen("folders open french"),
        ScreenCmd::Folders(FoldersCmd::Open("french".into()))
    );
    assert_eq!(
        screen("pdf open lecture.pdf"),
        ScreenCmd::PdfOpen("lecture.pdf".into())
    );
    assert_eq!(
        cmd("stack delete french"),
        Command::Stack(StackCmd::Delete("french".into()))
    );
}

#[test]
fn the_screen_words_refuse_what_they_do_not_name() {
    for bad in [
        "achievements sort",
        "graph now",
        "assistant send",
        "settings tab study",
        "match stop",
        "folders move x",
        "pdf move x",
        "stack delete",
    ] {
        assert!(parse_request(bad).is_err(), "{bad} should not parse");
    }
}

#[test]
fn reads_are_read_and_the_changes_are_write_except_stack_delete() {
    for line in [
        "achievements",
        "achievements filter global",
        "graph",
        "assistant",
        "settings",
        "match",
        "folders",
        "folders open french",
        "pdf open a.pdf",
    ] {
        assert_eq!(class_of(&cmd(line)), Class::Read, "{line}");
    }
    for line in [
        "assistant send hi",
        "settings slow-answer on",
        "match start",
    ] {
        assert_eq!(class_of(&cmd(line)), Class::Write, "{line}");
    }
    assert_eq!(class_of(&cmd("stack delete french")), Class::Destructive);
}

#[test]
fn every_new_command_has_a_help_row() {
    let r = ask(
        &mut engine(
            &sample_vault("ctl-help-screens").path,
            &temp_dir("ctl-help-screens-logs").path,
            policy_default(),
        ),
        "help",
    );
    let text = r["result"]["commands"].to_string();
    for usage in [
        "ui find",
        "ui click",
        "achievements filter",
        "graph",
        "assistant send",
        "settings slow-answer",
        "match start",
        "folders open",
        "pdf open",
        "stack delete",
    ] {
        assert!(text.contains(usage), "no help row for {usage}");
    }
}

#[test]
fn achievements_list_the_entries_with_their_progress() {
    let (vault, logs) = (sample_vault("ctl-ach"), temp_dir("ctl-ach-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "achievements");
    assert_eq!(r["ok"], true, "{r}");
    let entries = r["result"]["entries"].as_array().unwrap();
    assert!(!entries.is_empty(), "{r}");
    assert!(
        entries[0]["title"].is_string() && entries[0]["unlocked"].is_boolean(),
        "{r}"
    );
    let stacks = ask(&mut e, "achievements filter stack french");
    assert_eq!(stacks["ok"], true, "{stacks}");
    assert!(
        stacks["result"]["count"].as_u64().unwrap() <= entries.len() as u64,
        "{stacks}"
    );
}

#[test]
fn the_graph_answers_its_nodes_and_edges() {
    let (vault, logs) = (sample_vault("ctl-graph"), temp_dir("ctl-graph-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "graph");
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["result"]["nodes"], 4, "{r}");
    assert!(r["result"]["edges"].is_u64(), "{r}");
}

#[test]
fn the_assistant_without_a_model_answers_not_configured() {
    let (vault, logs) = (sample_vault("ctl-agent"), temp_dir("ctl-agent-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "assistant");
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["result"]["configured"], false, "{r}");
    let sent = ask(&mut e, "assistant send Explain recall");
    assert_eq!(sent["ok"], false, "{sent}");
    assert!(
        sent["error"].as_str().unwrap().contains("not configured"),
        "{sent}"
    );
}

#[test]
fn settings_list_the_sections_and_set_the_slow_answer_notice() {
    let (vault, logs) = (sample_vault("ctl-settings"), temp_dir("ctl-settings-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "settings");
    assert_eq!(r["ok"], true, "{r}");
    assert!(
        r["result"]["sections"].as_array().unwrap().len() >= 5,
        "{r}"
    );
    let off = ask(&mut e, "settings slow-answer off");
    assert_eq!(off["result"]["slow_answer_notice"], false, "{off}");
    let on = ask(&mut e, "settings slow-answer on");
    assert_eq!(on["result"]["slow_answer_notice"], true, "{on}");
}

#[test]
fn match_is_inactive_until_started_and_then_has_tiles() {
    let (vault, logs) = (sample_vault("ctl-match"), temp_dir("ctl-match-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    assert_eq!(ask(&mut e, "match")["result"]["active"], false);
    let started = ask(&mut e, "match start");
    assert_eq!(started["ok"], true, "{started}");
    assert_eq!(started["result"]["active"], true, "{started}");
    assert!(
        started["result"]["tiles"].as_u64().unwrap() >= 2,
        "{started}"
    );
}

#[test]
fn folders_list_the_entries_and_open_a_folder_with_its_children() {
    let (vault, logs) = (sample_vault("ctl-folders"), temp_dir("ctl-folders-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "folders");
    assert_eq!(r["ok"], true, "{r}");
    assert!(
        r["result"]["entries"]
            .to_string()
            .contains("\"path\":\"french\""),
        "{r}"
    );
    let opened = ask(&mut e, "folders open french");
    assert_eq!(opened["ok"], true, "{opened}");
    assert_eq!(opened["result"]["expanded"], true, "{opened}");
    assert!(
        opened["result"]["children"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "a.md"),
        "{opened}"
    );
    let missing = ask(&mut e, "folders open nowhere");
    assert!(
        missing["error"]
            .as_str()
            .unwrap()
            .contains("no file or folder"),
        "{missing}"
    );
}
