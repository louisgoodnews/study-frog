#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Create dialog laid out like Jira's: stack and type at the top, resizable editors, priority and difficulty.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{Modal, ModalMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn with_dialog() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    m
}

#[test]
fn the_dialog_starts_with_the_defaults_and_sends_what_was_chosen() {
    let mut m = with_dialog();
    let Some(Modal::Create(f)) = &m.state.modal else {
        panic!()
    };
    assert_eq!(
        (f.priority, f.difficulty),
        (Priority::Medium, Difficulty::Difficult)
    );
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(studyfrog_ui::modal::Field::Text, "q".into())),
    );
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(
            studyfrog_ui::modal::Field::Answer,
            "a".into(),
        )),
    );
    apply(&mut m, Msg::Modal(ModalMsg::SetPriority(Priority::Highest)));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::SetDifficulty(Difficulty::Easy)),
    );
    let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    let [Effect::CreateItem { choices, .. }] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(*choices, (Some(Priority::Highest), Some(Difficulty::Easy)));
}

#[test]
fn the_stack_and_the_type_are_in_the_header_and_the_editors_and_the_dialog_can_be_resized() {
    let mut h = harness(with_dialog());
    h.run_steps(8);
    assert_all_named(&h, "create dialog layout");
    for name in [
        "Parent stack",
        "Type of item",
        "Title",
        "Resize Front",
        "Resize Back",
        "Resize the dialog",
        "Enlarge the dialog",
        "More",
        "Priority",
        "Difficulty",
    ] {
        assert!(h.query_all_by_label(name).count() >= 1, "missing {name}");
    }
    h.get_by_label("Enlarge the dialog").click();
    h.run_steps(8);
    assert!(h.query_by_label("Shrink the dialog").is_some());
    h.get_by_label("Shrink the dialog").click();
    h.run_steps(8);
    assert!(h.query_by_label("Enlarge the dialog").is_some());
}
