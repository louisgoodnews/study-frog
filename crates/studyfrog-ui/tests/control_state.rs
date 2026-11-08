#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The control protocol's pure state serializer, command plans and answers.

mod common;

use serde_json::{Value, json};
use studyfrog_ui::control_cmd::{Command, ItemCmd};
use studyfrog_ui::control_help::{COMMANDS, help_json};
use studyfrog_ui::control_parse::parse_request;
use studyfrog_ui::control_plan::{plan_for, resolve_item, resolve_stack};
use studyfrog_ui::control_reply::{err_line, event_line, ok_line, result_for};
use studyfrog_ui::control_state::{screen_name, state_json};
use studyfrog_ui::control_words::screen_by_name;
use studyfrog_ui::msg::{EditAction, Msg};
use studyfrog_ui::state::{Notice, NoticeKind, Screen, State};

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

#[test]
fn an_empty_state_is_the_vault_chooser_with_nothing_open() {
    let v = state_json(&State::default());
    assert_eq!(v["screen"], "welcome");
    for key in [
        "vault", "editor", "modal", "rehearse", "library", "search", "notice",
    ] {
        assert_eq!(v[key], Value::Null, "{key}");
    }
    assert_eq!(v["notices"], json!([]));
    assert_eq!(v["history"], json!({"back": false, "forward": false}));
}

#[test]
fn every_screen_name_reads_back_as_the_same_screen() {
    use Screen::*;
    for s in [
        Welcome,
        Home,
        Folders,
        Agent,
        Prompts,
        Library,
        Rehearse,
        Settings,
        Graph,
        Calendar,
        Board,
        Achievements,
        Reports,
        Diary,
        Help,
    ] {
        assert_eq!(screen_by_name(screen_name(s)), Some(s), "{s:?}");
    }
}

#[test]
fn an_open_vault_is_described_with_its_stacks_and_the_screen_parts() {
    let (_dir, mut m, mut w) = common::flow::opened("ctl-state");
    let v = state_json(&m.state);
    assert_eq!(v["screen"], "home");
    assert_eq!(v["vault"]["stacks"][0]["folder"], "french");
    assert_eq!(v["vault"]["stacks"][0]["cards"], 2);
    assert_eq!(v["vault"]["inbox_items"], 1);
    common::flow::send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    let v = state_json(&m.state);
    assert_eq!(v["library"]["total"], 4);
    assert_eq!(v["library"]["rows"].as_array().unwrap().len(), 4);
    common::flow::send(&mut m, &mut w, Msg::OpenItem(studyfrog_core::uid::Uid(10)));
    let v = state_json(&m.state);
    assert_eq!(v["screen"], "editor");
    assert_eq!(v["editor"]["title"], "A");
    assert_eq!(
        v["editor"]["fields"][1]["text"].as_str().unwrap().trim(),
        "a"
    );
    assert_eq!(v["editor"]["dirty"], false);
}

#[test]
fn items_are_found_by_ulid_or_key_and_stacks_by_folder() {
    let (_dir, mut m, mut w) = common::flow::opened("ctl-resolve");
    assert_eq!(
        resolve_item(&m.state, "0000000000000000000000000A")
            .unwrap()
            .0,
        10
    );
    assert!(
        resolve_item(&m.state, "FLASHCARD_1")
            .unwrap_err()
            .contains("no item FLASHCARD_1")
    );
    common::flow::send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    m.state.library.items[0].row.key = Some("FLASHCARD_7".into());
    let uid = m.state.library.items[0].uid;
    assert_eq!(resolve_item(&m.state, "flashcard_7").unwrap(), uid);
    assert_eq!(resolve_stack(&m.state, "french/").unwrap().0, 1);
    assert!(
        resolve_stack(&m.state, "german")
            .unwrap_err()
            .contains("no stack")
    );
}

#[test]
fn commands_become_the_messages_the_screens_send() {
    let (_dir, m, _w) = common::flow::opened("ctl-plan");
    let s = &m.state;
    assert_eq!(
        plan_for(s, &cmd("nav board")).unwrap(),
        vec![vec![Msg::Navigate(Screen::Board)]]
    );
    let delete = plan_for(s, &cmd("item delete 0000000000000000000000000A")).unwrap();
    assert_eq!(delete.len(), 2, "the item is loaded before it is deleted");
    assert_eq!(delete[1][1], Msg::Edit(EditAction::ConfirmDelete));
    let edit = plan_for(s, &cmd("item edit 0000000000000000000000000A title New")).unwrap();
    assert_eq!(
        edit[1],
        vec![Msg::Edit(EditAction::Title("New".into())), Msg::SaveClicked]
    );
    assert!(plan_for(s, &cmd("item move 0000000000000000000000000A german")).is_err());
    assert!(plan_for(s, &cmd("rehearse setup --stack german")).is_err());
    assert_eq!(plan_for(s, &cmd("state")).unwrap(), Vec::<Vec<Msg>>::new());
    assert_eq!(plan_for(s, &cmd("item create note N")).unwrap()[0].len(), 8);
}

#[test]
fn a_new_error_notice_turns_the_answer_into_an_error() {
    let mut s = State::default();
    let before = s.notice.clone();
    s.notice = Some(Notice {
        kind: NoticeKind::Error,
        text: "Could not save".into(),
    });
    let open = Command::Item(ItemCmd::Open("x".into()));
    assert_eq!(
        result_for(&s, &open, &before).unwrap_err(),
        "Could not save"
    );
    assert!(
        result_for(&s, &Command::State, &before).is_ok(),
        "state always answers"
    );
    assert!(
        result_for(&s, &open, &s.notice.clone()).is_ok(),
        "an old notice is not this command's"
    );
}

#[test]
fn answers_and_events_are_single_json_lines() {
    let ok: Value = serde_json::from_str(&ok_line(&json!(4), json!({"a": 1}))).unwrap();
    assert_eq!(ok, json!({"id": 4, "ok": true, "result": {"a": 1}}));
    let err: Value = serde_json::from_str(&err_line(&json!(null), "no")).unwrap();
    assert_eq!(err, json!({"id": null, "ok": false, "error": "no"}));
    assert!(!event_line("log", json!({"m": "a\nb"})).contains('\n'));
    let help = help_json();
    assert_eq!(help["commands"].as_array().unwrap().len(), COMMANDS.len());
    assert!(
        COMMANDS
            .iter()
            .all(|(_, _, class)| ["read", "write", "destructive"].contains(class))
    );
}
