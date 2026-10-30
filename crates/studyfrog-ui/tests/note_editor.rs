#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Markdown editor inside the Create dialog.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_service::create_more::NewKind;
use studyfrog_ui::modal::{Modal, ModalMsg};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

fn open(kind: NewKind) -> studyfrog_ui::model::Model {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    apply(&mut m, Msg::Modal(ModalMsg::SetKind(kind)));
    m
}

#[test]
fn a_new_note_opens_a_full_markdown_editor_with_a_toolbar_and_a_live_preview() {
    let mut h = harness(open(NewKind::Note));
    h.run_steps(2);
    for name in [
        "Bold in Body",
        "Bullet list in Body",
        "Link to a note in Body",
        "Body: split",
        "Body: preview",
    ] {
        assert!(h.query_by_label(name).is_some(), "missing {name}");
    }
    h.get_by_label("Body").focus();
    h.get_by_label("Body").type_text("# Hello");
    h.run_steps(2);
    let Some(Modal::Create(f)) = &h.state().0.state.modal else {
        panic!()
    };
    assert_eq!(f.text, "# Hello");
    h.get_by_label("Bold in Body").click();
    h.run_steps(2);
    let Some(Modal::Create(f)) = &h.state().0.state.modal else {
        panic!()
    };
    assert!(f.text.contains("**"), "{:?}", f.text);
    assert_all_named(&h, "note editor");
}

#[test]
fn cards_and_questions_get_the_toolbar_on_their_text_areas_too() {
    let mut h = harness(open(NewKind::Card));
    h.run_steps(2);
    assert!(
        h.query_by_label("Italic in Front").is_some()
            && h.query_by_label("Italic in Back").is_some()
    );
    assert_all_named(&h, "card editor");
}
