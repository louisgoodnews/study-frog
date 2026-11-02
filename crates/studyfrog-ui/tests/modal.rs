#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The create dialog, the vault dialog and the first-launch toast (update logic).

mod common;

use common::views::open_model;
use studyfrog_service::create_more::{Created, NewKind};
use studyfrog_store::overview::VaultOverview;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{CreateOutcome, Field, Modal, ModalMsg, problem};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};

fn modal(m: &mut Model, msg: ModalMsg) -> Vec<Effect> {
    apply(m, Msg::Modal(msg))
}

fn form(m: &Model) -> studyfrog_ui::modal::CreateForm {
    match &m.state.modal {
        Some(Modal::Create(f)) => f.clone(),
        other => panic!("no create dialog: {other:?}"),
    }
}

fn outcome(path: &str, title: &str, m: &Model) -> Result<Box<CreateOutcome>, String> {
    Ok(Box::new(CreateOutcome {
        info: m.state.vault.clone().unwrap(),
        created: Created {
            title: title.into(),
            path: path.into(),
        },
        warnings: Vec::new(),
    }))
}

#[test]
fn the_dialog_starts_on_a_card_in_the_open_stack_or_on_a_stack_when_there_are_none() {
    let mut m = open_model();
    modal(&mut m, ModalMsg::OpenCreate);
    let f = form(&m);
    assert_eq!(
        (f.kind, f.parent.as_deref()),
        (NewKind::Card, Some("french")),
        "the first stack when none is open"
    );
    apply(&mut m, Msg::Select(Selection::Stack("maths".into())));
    modal(&mut m, ModalMsg::OpenCreate);
    assert_eq!(
        form(&m).parent.as_deref(),
        Some("maths"),
        "the stack that is open"
    );
    modal(&mut m, ModalMsg::OpenCreateStack);
    assert_eq!(form(&m).kind, NewKind::Stack);
    modal(&mut m, ModalMsg::Close);
    assert!(m.state.modal.is_none());

    let mut empty = open_model();
    empty.state.vault.as_mut().unwrap().overview = VaultOverview::default();
    modal(&mut empty, ModalMsg::OpenCreate);
    let f = form(&empty);
    assert_eq!(
        (f.kind, f.parent),
        (NewKind::Stack, None),
        "with no stack there is only a stack to make"
    );
}

#[test]
fn every_kind_says_what_is_missing_and_everything_but_a_stack_needs_a_parent() {
    let mut m = open_model();
    modal(&mut m, ModalMsg::OpenCreate);
    let mut f = form(&m);
    assert!(problem(&f).unwrap().contains("front and a back"));
    f.text = "Q?".into();
    f.answer = "A".into();
    assert_eq!(problem(&f), None);
    f.parent = None;
    assert!(problem(&f).unwrap().contains("Choose the stack"));
    f.kind = NewKind::Note;
    f.parent = Some("french".into());
    assert!(problem(&f).unwrap().contains("title"));
    f.title = "Plan".into();
    assert_eq!(problem(&f), None, "a note needs a title, not a body");
    f.kind = NewKind::Question;
    f.answer.clear();
    assert!(
        problem(&f)
            .unwrap()
            .contains("prompt and a reference answer")
    );
    f.kind = NewKind::Stack;
    f.parent = None;
    assert_eq!(problem(&f), None, "a top-level stack with a title is fine");
    f.title = " ".into();
    assert!(problem(&f).unwrap().contains("title"));
}

#[test]
fn submitting_sends_one_request_with_clean_values_and_a_second_press_waits() {
    let mut m = open_model();
    modal(&mut m, ModalMsg::OpenCreate);
    assert!(
        modal(&mut m, ModalMsg::Submit).is_empty(),
        "nothing typed yet"
    );
    assert!(form(&m).error.is_some());
    modal(&mut m, ModalMsg::Edit(Field::Text, "Bonjour?".into()));
    assert!(form(&m).error.is_none(), "typing clears the complaint");
    modal(&mut m, ModalMsg::Edit(Field::Answer, "Hello".into()));
    modal(&mut m, ModalMsg::Edit(Field::Title, "  Greeting ".into()));
    modal(&mut m, ModalMsg::Edit(Field::Tags, "geo, ,easy ".into()));
    let fx = modal(&mut m, ModalMsg::Submit);
    assert_eq!(
        fx,
        [Effect::CreateItem {
            kind: NewKind::Card,
            title: "Greeting".into(),
            text: "Bonjour?".into(),
            answer: "Hello".into(),
            tags: vec!["geo".into(), "easy".into()],
            parent: Some("french".into()),
            due: None,
        }]
    );
    assert!(form(&m).busy);
    assert!(
        modal(&mut m, ModalMsg::Submit).is_empty(),
        "no second request while one is running"
    );
}

#[test]
fn a_finished_creation_closes_the_dialog_or_keeps_it_for_the_next_one() {
    let mut m = open_model();
    modal(&mut m, ModalMsg::OpenCreate);
    modal(&mut m, ModalMsg::Edit(Field::Text, "Q?".into()));
    modal(&mut m, ModalMsg::Edit(Field::Answer, "A".into()));
    modal(&mut m, ModalMsg::Edit(Field::Tags, "t".into()));
    modal(&mut m, ModalMsg::SetAnother(true));
    modal(&mut m, ModalMsg::Submit);
    let fx = {
        let o = outcome("french/q.md", "Q?", &m);
        modal(&mut m, ModalMsg::Created(o))
    };
    assert!(fx.contains(&Effect::LoadBoard) && fx.contains(&Effect::LoadSessions));
    let f = form(&m);
    assert_eq!(
        (f.text.as_str(), f.answer.as_str(), f.title.as_str(), f.busy),
        ("", "", "", false),
        "fields are cleared"
    );
    assert_eq!(
        (f.kind, f.parent.as_deref(), f.tags.as_str(), f.another),
        (NewKind::Card, Some("french"), "t", true),
        "kind, stack and tags stay"
    );
    assert_eq!(m.state.notice.as_ref().unwrap().text, "Created \"Q?\".");

    modal(&mut m, ModalMsg::SetAnother(false));
    modal(&mut m, ModalMsg::Edit(Field::Text, "Q2?".into()));
    modal(&mut m, ModalMsg::Edit(Field::Answer, "A2".into()));
    modal(&mut m, ModalMsg::Submit);
    {
        let o = outcome("french/q2.md", "Q2?", &m);
        modal(&mut m, ModalMsg::Created(o))
    };
    assert!(
        m.state.modal.is_none(),
        "without 'create another' the dialog closes"
    );
}

#[test]
fn a_new_stack_is_opened_and_a_refusal_keeps_the_form_with_its_reason() {
    let mut m = open_model();
    modal(&mut m, ModalMsg::OpenCreateStack);
    modal(&mut m, ModalMsg::Edit(Field::Title, "Verbs".into()));
    modal(&mut m, ModalMsg::Submit);
    modal(
        &mut m,
        ModalMsg::Created(Err("that folder is not a stack".into())),
    );
    let f = form(&m);
    assert_eq!(
        (f.busy, f.error.as_deref(), f.title.as_str()),
        (false, Some("that folder is not a stack"), "Verbs")
    );

    modal(&mut m, ModalMsg::Submit);
    let fx = {
        let o = outcome("french/verbs/_stack.md", "Verbs", &m);
        modal(&mut m, ModalMsg::Created(o))
    };
    assert_eq!(
        m.state.browser.selection,
        Selection::Stack("french/verbs".into())
    );
    assert_eq!(m.state.screen, Screen::Library);
    assert_eq!(
        fx,
        [
            Effect::LoadItems(Selection::Stack("french/verbs".into())),
            Effect::LoadStatusCounts(Selection::Stack("french/verbs".into())),
            Effect::LoadBoard,
            Effect::LoadSessions
        ]
    );
}
