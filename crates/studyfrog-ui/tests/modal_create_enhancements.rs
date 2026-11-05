#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Integration tests for Milestone 4: Create Menu Due Date/Priority & Note Modal Expansion (R3 & R4).

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::civil::date_new;
use studyfrog_core::priority::Priority;
use studyfrog_service::create_more::NewKind;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal::{Field, Modal, ModalMsg, problem};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::view::modal_size::{NORMAL, widest, width};

fn dialog_model(kind: Option<NewKind>) -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    if let Some(k) = kind {
        apply(&mut m, Msg::Modal(ModalMsg::SetKind(k)));
    }
    m
}

#[test]
fn setting_priority_on_stack_creation_sets_choice_and_renders_in_ui() {
    let mut m = dialog_model(Some(NewKind::Stack));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Title, "Calculus".into())),
    );
    apply(&mut m, Msg::Modal(ModalMsg::SetPriority(Priority::Highest)));
    let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    let [Effect::CreateItem { choices, kind, .. }] = &fx[..] else {
        panic!("expected CreateItem effect: {fx:?}");
    };
    assert_eq!(*kind, NewKind::Stack);
    assert_eq!(*choices, (Some(Priority::Highest), None));

    let mut h = harness(dialog_model(Some(NewKind::Stack)));
    h.run_steps(4);
    assert!(
        h.query_all_by_label("Priority").count() >= 2,
        "missing Priority in dialog"
    );
}

#[test]
fn setting_due_date_on_card_note_and_question_creation() {
    for kind in [NewKind::Card, NewKind::Note, NewKind::Question] {
        let mut m = dialog_model(Some(kind));
        apply(
            &mut m,
            Msg::Modal(ModalMsg::Edit(Field::Title, "Topic".into())),
        );
        apply(
            &mut m,
            Msg::Modal(ModalMsg::Edit(Field::Text, "Front/Body".into())),
        );
        apply(
            &mut m,
            Msg::Modal(ModalMsg::Edit(Field::Answer, "Back/Ans".into())),
        );
        apply(
            &mut m,
            Msg::Modal(ModalMsg::Edit(Field::Due, "2026-12-31".into())),
        );
        let fx = apply(&mut m, Msg::Modal(ModalMsg::Submit));
        let [Effect::CreateItem { due, kind: k, .. }] = &fx[..] else {
            panic!("expected CreateItem for {kind:?}: {fx:?}");
        };
        assert_eq!(*k, kind);
        let due = due.as_ref().expect("due date should be parsed");
        assert_eq!(due.date, date_new(2026, 12, 31).unwrap());
    }

    let mut m = dialog_model(Some(NewKind::Card));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Due, "not-a-date".into())),
    );
    let Some(Modal::Create(f)) = &m.state.modal else {
        panic!()
    };
    assert_eq!(problem(f), Some("The due date must look like 2026-12-31."));
}

#[test]
fn auto_expansion_to_full_window_when_kind_is_note_and_default_write_view() {
    let mut h = harness(dialog_model(Some(NewKind::Card)));
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Card), NORMAL);

    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Note)),
    );
    h.run_steps(4);
    let full = widest(&h.ctx);
    assert_eq!(width(&h.ctx, NewKind::Note), full);
    assert!(full > NORMAL + 100.0, "widest should be larger than normal");

    assert!(h.query_by_label("Body: write").is_some());
    assert!(h.query_by_label("Body: split").is_some());
    assert!(h.query_by_label("Body: preview").is_some());
    assert!(h.query_by_label("Body").is_some());
}

#[test]
fn shrink_button_functionality_and_manual_resize_support() {
    let mut h = harness(dialog_model(Some(NewKind::Note)));
    h.run_steps(4);
    let full = widest(&h.ctx);
    assert_eq!(width(&h.ctx, NewKind::Note), full);

    assert!(h.query_by_label("Shrink the dialog").is_some());
    h.get_by_label("Shrink the dialog").click();
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Note), NORMAL);

    assert!(h.query_by_label("Enlarge the dialog").is_some());
    h.get_by_label("Enlarge the dialog").click();
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Note), full);

    studyfrog_ui::view::modal_size::set_width(&h.ctx, NewKind::Note, 750.0);
    assert_eq!(width(&h.ctx, NewKind::Note), 750.0);

    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Card)),
    );
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Card), NORMAL);

    apply(
        &mut h.state_mut().0,
        Msg::Modal(ModalMsg::SetKind(NewKind::Note)),
    );
    h.run_steps(4);
    assert_eq!(width(&h.ctx, NewKind::Note), 750.0);

    studyfrog_ui::view::modal_size::reset(&h.ctx);
    assert_eq!(width(&h.ctx, NewKind::Note), full);
}
