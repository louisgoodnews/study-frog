#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The board commands of the control protocol: words, class, and the answers of the columns, one
//! column, and a card added to a named column (STORY-0124, EPIC-064).

mod common;
mod control_common;

use control_common::*;
use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_cmd::board::BoardCmd;
use studyfrog_ui::control_cmd::{Class, class_of};
use studyfrog_ui::control_parse::parse_request;

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

#[test]
fn board_words_parse() {
    assert_eq!(cmd("board"), Command::Board(BoardCmd::List));
    assert_eq!(cmd("board list"), Command::Board(BoardCmd::List));
    assert_eq!(
        cmd("board column To Do"),
        Command::Board(BoardCmd::Column("To Do".into()))
    );
    assert_eq!(
        cmd("board add \"Read chapter 3\" --column Doing"),
        Command::Board(BoardCmd::Add {
            title: "Read chapter 3".into(),
            column: Some("Doing".into()),
        })
    );
    assert_eq!(
        cmd("board add Plain"),
        Command::Board(BoardCmd::Add {
            title: "Plain".into(),
            column: None,
        })
    );
    assert!(parse_request("board column").is_err());
    assert!(parse_request("board add").is_err());
    assert!(parse_request("board move").is_err());
}

#[test]
fn reading_the_board_is_read_and_adding_a_card_is_write() {
    assert_eq!(class_of(&cmd("board")), Class::Read);
    assert_eq!(class_of(&cmd("board column Done")), Class::Read);
    assert_eq!(class_of(&cmd("board add Card")), Class::Write);
}

#[test]
fn the_board_lists_its_columns_and_their_cards() {
    let (vault, logs) = (sample_vault("ctl-board"), temp_dir("ctl-board-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "board");
    assert_eq!(r["ok"], true, "{r}");
    let columns = r["result"]["columns"].as_array().unwrap();
    assert!(columns.len() >= 2, "{r}");
    assert!(columns[0]["cards"].is_array(), "{r}");
    let name = columns[1]["name"].as_str().unwrap();

    let one = ask(&mut e, &format!("board column \"{name}\""));
    assert_eq!(one["ok"], true, "{one}");
    assert_eq!(one["result"]["name"], name, "{one}");

    let missing = ask(&mut e, "board column Nowhere");
    assert_eq!(missing["ok"], false, "{missing}");
    assert!(
        missing["error"]
            .as_str()
            .unwrap()
            .contains("no column named")
    );
}

#[test]
fn a_card_goes_into_the_named_column_and_its_status_is_that_column() {
    let (vault, logs) = (
        sample_vault("ctl-board-add"),
        temp_dir("ctl-board-add-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "board");
    let name = r["result"]["columns"][1]["name"]
        .as_str()
        .unwrap()
        .to_owned();

    let added = ask(
        &mut e,
        &format!("board add \"Control card\" --column \"{name}\""),
    );
    assert_eq!(added["ok"], true, "{added}");
    assert_eq!(added["result"]["column"], name.as_str(), "{added}");
    assert_eq!(added["result"]["card"]["title"], "Control card", "{added}");
    assert_eq!(added["result"]["status"], name.as_str(), "{added}");

    let read = ask(&mut e, &format!("board column \"{name}\""));
    let cards = read["result"]["cards"].as_array().unwrap();
    let card = cards
        .iter()
        .find(|c| c["title"] == "Control card")
        .unwrap_or_else(|| panic!("{read}"));
    assert_eq!(card["status"], name.as_str(), "{read}");
}

#[test]
fn a_card_for_a_column_that_is_not_there_is_refused_with_the_reason() {
    let (vault, logs) = (
        sample_vault("ctl-board-none"),
        temp_dir("ctl-board-none-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let r = ask(&mut e, "board add \"Lost card\" --column Nowhere");
    assert_eq!(r["ok"], false, "{r}");
    assert!(
        r["error"]
            .as_str()
            .unwrap()
            .contains("No column named Nowhere"),
        "{r}"
    );
}

#[test]
fn a_read_only_policy_refuses_a_card() {
    let (vault, logs) = (sample_vault("ctl-board-ro"), temp_dir("ctl-board-ro-logs"));
    let read_only = ControlPolicy {
        allow_write: false,
        allow_destructive: false,
    };
    let mut e = engine(&vault.path, &logs.path, read_only);
    let r = ask(&mut e, "board add Refused");
    assert_eq!(r["ok"], false, "{r}");
    assert!(
        r["error"]
            .as_str()
            .unwrap()
            .contains("writing is switched off")
    );
}
