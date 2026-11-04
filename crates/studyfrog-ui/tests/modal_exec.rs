#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Creating items through the executor against a real vault.

mod common;

use common::{NOW_NS, sample_vault};
use studyfrog_service::create_more::NewKind;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::modal::ModalMsg;
use studyfrog_ui::msg::Msg;

#[test]
fn the_executor_creates_nested_stacks_and_items_and_reports_refusals() {
    let vault = sample_vault("modal-exec");
    let mut w = worker_new(None);
    execute(
        &mut w,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let make =
        |kind, title: &str, text: &str, answer: &str, parent: Option<&str>| Effect::CreateItem {
            kind,
            title: title.into(),
            text: text.into(),
            answer: answer.into(),
            tags: vec![],
            parent: parent.map(str::to_owned),
            due: None,
            also_in: None,
            choices: (None, None),
        };
    let msgs = execute(
        &mut w,
        make(NewKind::Stack, "Verbs", "", "", Some("french")),
        NOW_NS,
    );
    let Msg::Modal(ModalMsg::Created(Ok(out))) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!(out.created.path, "french/verbs/_stack.md");
    let verbs = out
        .info
        .overview
        .stacks
        .iter()
        .find(|s| s.folder == "french/verbs")
        .unwrap();
    assert_eq!(verbs.parent.as_deref(), Some("french"));

    let msgs = execute(
        &mut w,
        make(NewKind::Card, "", "Je suis?", "I am", Some("french/verbs")),
        NOW_NS,
    );
    assert!(
        matches!(&msgs[0], Msg::Modal(ModalMsg::Created(Ok(o))) if o.created.path == "french/verbs/je-suis.md")
    );
    let msgs = execute(&mut w, make(NewKind::Note, "Loose", "x", "", None), NOW_NS);
    assert!(
        matches!(&msgs[0], Msg::Modal(ModalMsg::Created(Err(e))) if e.contains("choose the stack")),
        "{:?}",
        msgs[0]
    );
}
