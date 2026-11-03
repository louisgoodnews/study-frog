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

fn rect_of(h: &egui_kittest::Harness<'_, common::views::App>, label: &str) -> eframe::egui::Rect {
    use egui_kittest::kittest::NodeT;
    let b = h
        .get_by_label(label)
        .accesskit_node()
        .bounding_box()
        .unwrap();
    eframe::egui::Rect::from_min_max(
        eframe::egui::pos2(b.x0 as f32, b.y0 as f32),
        eframe::egui::pos2(b.x1 as f32, b.y1 as f32),
    )
}

#[test]
fn at_the_normal_width_no_header_control_overlaps_another() {
    let mut m = with_dialog();
    m.state.vault.as_mut().unwrap().overview.stacks[0].title =
        "Psychologische Diagnostik und Evaluation".into();
    let mut h = harness(m);
    h.run_steps(8);
    let names = [
        "Parent stack",
        "Type of item",
        "Enlarge the dialog",
        "More",
        "Close",
    ];
    let rects: Vec<_> = names.iter().map(|n| (*n, rect_of(&h, n))).collect();
    for (i, (a, ra)) in rects.iter().enumerate() {
        for (b, rb) in &rects[i + 1..] {
            assert!(!ra.intersects(*rb), "{a} {ra:?} overlaps {b} {rb:?}");
        }
    }
    assert!(
        rect_of(&h, "Type of item").width() >= 90.0,
        "the type name is not cut"
    );
}

#[test]
fn closing_a_filled_dialog_asks_first_and_an_untouched_one_closes_at_once() {
    use studyfrog_ui::modal::Modal;
    let mut m = with_dialog();
    apply(&mut m, Msg::Modal(ModalMsg::Close));
    assert!(m.state.modal.is_none(), "nothing written: no question");

    for change in [
        ModalMsg::Edit(studyfrog_ui::modal::Field::Title, "x".into()),
        ModalMsg::Edit(studyfrog_ui::modal::Field::Text, "front".into()),
        ModalMsg::Edit(studyfrog_ui::modal::Field::Tags, "a".into()),
        ModalMsg::SetPriority(Priority::High),
        ModalMsg::SetDifficulty(Difficulty::Easy),
    ] {
        let mut m = with_dialog();
        apply(&mut m, Msg::Modal(change.clone()));
        apply(&mut m, Msg::Modal(ModalMsg::Close));
        let Some(Modal::Create(f)) = &m.state.modal else {
            panic!("{change:?} closed the dialog")
        };
        assert!(f.confirm_discard, "{change:?} asks first");
    }

    let mut m = with_dialog();
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(studyfrog_ui::modal::Field::Text, "t".into())),
    );
    apply(&mut m, Msg::Modal(ModalMsg::Close));
    apply(&mut m, Msg::Modal(ModalMsg::KeepEditing));
    let Some(Modal::Create(f)) = &m.state.modal else {
        panic!("closed")
    };
    assert!(
        !f.confirm_discard && f.text == "t",
        "keeps what was written"
    );
    apply(&mut m, Msg::Modal(ModalMsg::Close));
    apply(&mut m, Msg::Modal(ModalMsg::Close));
    let Some(Modal::Create(f)) = &m.state.modal else {
        panic!("closed")
    };
    assert!(!f.confirm_discard, "Escape on the question withdraws it");
    apply(&mut m, Msg::Modal(ModalMsg::Close));
    apply(&mut m, Msg::Modal(ModalMsg::Discard));
    assert!(m.state.modal.is_none());
}

#[test]
fn the_question_shows_two_named_buttons_in_the_dialog() {
    let mut m = with_dialog();
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(studyfrog_ui::modal::Field::Text, "t".into())),
    );
    apply(&mut m, Msg::Modal(ModalMsg::Close));
    let mut h = harness(m);
    h.run_steps(8);
    assert!(
        h.query_by_label_contains("Discard what you wrote?")
            .is_some()
    );
    h.get_by_label("Keep editing").click();
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Discard what you wrote?")
            .is_none()
    );
}
