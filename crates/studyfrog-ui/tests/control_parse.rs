#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The control protocol's pure parser and policy: words and JSON to commands.

use serde_json::json;
use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_service::create_more::NewKind;
use studyfrog_ui::control_cmd::*;
use studyfrog_ui::control_parse::{parse_request, parse_words, tokenize};
use studyfrog_ui::setup::Preset;
use studyfrog_ui::state::Screen;
use studyfrog_ui::transfer_types::Format;

fn cmd(line: &str) -> Command {
    parse_request(line).unwrap().cmd
}

fn err(line: &str) -> String {
    parse_request(line).unwrap_err().1
}

#[test]
fn words_are_split_by_spaces_quotes_and_escapes() {
    assert_eq!(
        tokenize(r#"a "b c"  d\ e "" f"#).unwrap(),
        ["a", "b c", "d e", "", "f"]
    );
    assert!(tokenize(r#"a "b"#).unwrap_err().contains("quote"));
}

#[test]
fn navigation_and_the_simple_commands() {
    assert_eq!(cmd("nav home"), Command::Nav(Screen::Home));
    assert_eq!(cmd("nav items"), Command::Nav(Screen::Library));
    assert_eq!(cmd("nav assistant"), Command::Nav(Screen::Agent));
    assert_eq!(cmd("nav back"), Command::Back);
    assert_eq!(cmd("forward"), Command::Forward);
    for (line, c) in [
        ("state", Command::State),
        ("help", Command::Help),
        ("notices", Command::Notices),
    ] {
        assert_eq!(cmd(line), c);
    }
    assert_eq!(cmd("settle"), Command::Settle);
    assert!(err("nav nowhere").contains("unknown screen nowhere"));
    assert!(err("frobnicate").contains("unknown command frobnicate"));
}

#[test]
fn item_commands_take_their_fields_and_flags() {
    let created =
        cmd(r#"item create card "Two words" --text "front side" --answer back --stack french"#);
    let Command::Item(ItemCmd::Create(a)) = created else {
        panic!("{created:?}")
    };
    assert_eq!(
        (a.kind, a.title.as_str(), a.text.as_str()),
        (NewKind::Card, "Two words", "front side")
    );
    assert_eq!(
        (a.answer.as_str(), a.stack.as_deref()),
        ("back", Some("french"))
    );
    assert_eq!(
        cmd("item edit FLASHCARD_1 back new answer text"),
        Command::Item(ItemCmd::Edit(
            "FLASHCARD_1".into(),
            EditTarget::Field(1),
            "new answer text".into()
        ))
    );
    assert_eq!(
        cmd("item edit X field2 t"),
        Command::Item(ItemCmd::Edit("X".into(), EditTarget::Field(2), "t".into()))
    );
    assert_eq!(
        cmd("item color X none"),
        Command::Item(ItemCmd::Color("X".into(), None))
    );
    assert_eq!(
        cmd("item move X bio"),
        Command::Item(ItemCmd::Move("X".into(), "bio".into()))
    );
    assert!(err("item create box T").contains("unknown kind box"));
    assert!(err("item edit X").contains("missing field"));
    assert!(err("item create card T --stack").contains("--stack needs a value"));
}

#[test]
fn stack_and_rehearse_commands() {
    assert_eq!(
        cmd("stack create Bio --parent sci"),
        Command::Stack(StackCmd::Create("Bio".into(), Some("sci".into())))
    );
    assert_eq!(
        cmd("stack move a"),
        Command::Stack(StackCmd::Move("a".into(), String::new()))
    );
    let setup = cmd("rehearse setup --preset all --mode answer --limit 5 --stack a --stack b");
    let Command::Rehearse(RehearseCmd::Setup(s)) = setup else {
        panic!()
    };
    assert_eq!(
        (s.preset, s.mode, s.limit.as_deref()),
        (
            Some(Preset::EverythingInOrder),
            Some(RunMode::Answer),
            Some("5")
        )
    );
    assert_eq!(s.stacks, ["a", "b"]);
    assert_eq!(
        cmd("rehearse rate 3"),
        Command::Rehearse(RehearseCmd::Rate(Rating::Good))
    );
    assert_eq!(
        cmd("rehearse mark hard"),
        Command::Rehearse(RehearseCmd::Mark(Difficulty::Difficult))
    );
    assert_eq!(
        cmd("rehearse answer la vie"),
        Command::Rehearse(RehearseCmd::Answer("la vie".into()))
    );
    assert!(err("rehearse rate great").contains("unknown rating"));
    assert!(err("rehearse").contains("unknown rehearse step"));
}

#[test]
fn query_and_search_keep_their_text_as_written() {
    assert_eq!(
        cmd(r#"query title ~ "a b"  and type = card"#),
        Command::Query(r#"title ~ "a b"  and type = card"#.into())
    );
    assert_eq!(
        parse_words("search  le chat ").unwrap(),
        Command::Search("le chat".into())
    );
    let j = json!({"id": 3, "cmd": "query", "args": ["type = card", "order by title"]}).to_string();
    assert_eq!(cmd(&j), Command::Query("type = card order by title".into()));
}

#[test]
fn transfer_logs_and_screenshot() {
    let e = cmd("export json /tmp/out --stack french --scheduling");
    assert_eq!(
        e,
        Command::Export(ExportArgs {
            format: Format::Json,
            dest: "/tmp/out".into(),
            stack: Some("french".into()),
            scheduling: true
        })
    );
    assert_eq!(
        cmd("import a.csv"),
        Command::Import {
            path: "a.csv".into(),
            stack: None
        }
    );
    assert_eq!(
        cmd("logs"),
        Command::Logs(LogsQuery {
            tail: 50,
            follow: false,
            level: None
        })
    );
    assert_eq!(
        cmd("logs --tail 5 --follow --level warn"),
        Command::Logs(LogsQuery {
            tail: 5,
            follow: true,
            level: Some("warn".into())
        })
    );
    assert!(err("logs --level loud").contains("unknown level"));
    assert_eq!(
        cmd("screenshot /tmp/a.png"),
        Command::Screenshot("/tmp/a.png".into())
    );
}

#[test]
fn json_requests_carry_their_id_even_when_they_fail() {
    let r = parse_request(r#"{"id": "x1", "cmd": "nav", "args": ["home"]}"#).unwrap();
    assert_eq!((r.id, r.cmd), (json!("x1"), Command::Nav(Screen::Home)));
    let r = parse_request(r#"{"id": 2, "cmd": "item open FLASHCARD_2"}"#).unwrap();
    assert_eq!(r.cmd, Command::Item(ItemCmd::Open("FLASHCARD_2".into())));
    assert_eq!(
        parse_request(r#"{"id": 9, "cmd": "nav", "args": ["x"]}"#)
            .unwrap_err()
            .0,
        json!(9)
    );
    assert!(err(r#"{"id": 9}"#).contains("no \"cmd\""));
    assert!(err("{nope").contains("bad JSON"));
    assert_eq!(parse_request("state").unwrap().id, json!(null));
}

#[test]
fn the_default_policy_lets_writes_through_and_refuses_destructive_commands() {
    let p = policy_default();
    assert_eq!(class_of(&cmd("item delete X")), Class::Destructive);
    assert_eq!(class_of(&cmd("item open X")), Class::Read);
    assert_eq!(class_of(&cmd("rehearse rate good")), Class::Write);
    assert!(allowed(&p, &cmd("item create note T")).is_ok());
    assert!(
        allowed(&p, &cmd("item delete X"))
            .unwrap_err()
            .contains("--control-allow-destructive")
    );
    let read_only = ControlPolicy {
        allow_write: false,
        allow_destructive: false,
    };
    assert!(
        allowed(&read_only, &cmd("stack create S"))
            .unwrap_err()
            .contains("read-only")
    );
    assert!(allowed(&read_only, &cmd("query type = card")).is_ok());
    let all = ControlPolicy {
        allow_write: true,
        allow_destructive: true,
    };
    assert!(allowed(&all, &cmd("item delete X")).is_ok());
}
