#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless egui_kittest suite for the mock exam item format selection modal (STORY-0207).
use studyfrog_ui::msg::MsgVault;

mod common;

use common::views::{assert_all_named, harness, open_model};
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_rehearsal::exam_format::{
    ALL_EXAM_FORMATS, ExamItemFormat, MixMode, format_index, format_label,
};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal_types::Modal;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::mock_exam::format_modal::FormatModalMsg;
use studyfrog_ui::setup::mock_exam::{ExamSource, MockExamEffect, MockExamMsg, SlideChoice};
use studyfrog_ui::state::{Screen, Selection};

fn setup_rehearse() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.browser.selection = Selection::Stack("french".to_owned());
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
    m
}

fn assert_checkboxes_named(h: &egui_kittest::Harness<'_, common::views::App>) {
    for node in h.query_all_by_role(Role::CheckBox) {
        let label = node.accesskit_node().label().unwrap_or_default();
        assert!(!label.trim().is_empty(), "CheckBox missing label");
    }
}

#[test]
fn clicking_mock_exam_opens_format_modal_and_validates_selection() {
    let mut h = harness(setup_rehearse());
    h.run_steps(4);
    h.get_by_label("Mock exam").click();
    h.run_steps(4);
    assert!(matches!(
        h.state().0.state.modal,
        Some(Modal::MockExamFormat(_))
    ));
    assert_all_named(&h, "mock exam format modal");
    assert_checkboxes_named(&h);

    for fmt in ALL_EXAM_FORMATS {
        let lbl = format!("Include {}", format_label(fmt));
        h.get_by_label(&lbl).click();
        h.run_steps(2);
    }
    assert!(
        h.query_by_label("Select at least one item format.")
            .is_some()
    );

    h.get_by_label("Include Fill in the gaps").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("Select at least one item format.")
            .is_none()
    );

    h.get_by_label("Mix by exact counts").click();
    h.run_steps(2);
    h.get_by_label("Increase Fill in the gaps").click();
    h.run_steps(2);
    h.get_by_label("Generate exam").click();
    h.run_steps(4);

    assert!(h.state().0.state.modal.is_none());
    let effects = &h.state().1;
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::MockExam(MockExamEffect::Generate {
                source: ExamSource::Notes(Some(f)),
                mix,
                ..
            }) if f == "french"
                && mix.mode == MixMode::Counts
                && mix.formats[format_index(ExamItemFormat::FillInGaps)].enabled
        )),
        "{effects:?}"
    );
}

#[test]
fn format_preferences_persist_across_modal_opens_and_cancel_discards_edits() {
    let mut m = setup_rehearse();
    apply(&mut m, Msg::MockExam(MockExamMsg::Open));
    apply(
        &mut m,
        Msg::MockExam(MockExamMsg::FormatModal(FormatModalMsg::SetMode(
            MixMode::Counts,
        ))),
    );
    apply(
        &mut m,
        Msg::MockExam(MockExamMsg::FormatModal(FormatModalMsg::SetValue(
            ExamItemFormat::SingleSelect,
            4,
        ))),
    );
    apply(
        &mut m,
        Msg::MockExam(MockExamMsg::FormatModal(FormatModalMsg::Confirm)),
    );
    apply(&mut m, Msg::MockExam(MockExamMsg::Close));

    let mut h = harness(m);
    h.run_steps(4);
    h.get_by_label("Mock exam").click();
    h.run_steps(4);
    let Some(Modal::MockExamFormat(modal)) = &h.state().0.state.modal else {
        panic!("expected modal");
    };
    assert_eq!(modal.mix.mode, MixMode::Counts);
    assert_eq!(
        modal.mix.formats[format_index(ExamItemFormat::SingleSelect)].value,
        4
    );

    h.get_by_label("Reset defaults").click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.state().0.state.modal.is_none());
    assert_eq!(
        h.state().0.state.rehearse.mock_format_prefs.mode,
        MixMode::Counts
    );
}

#[test]
fn mock_exam_from_pdf_opens_format_modal_after_picking_slide() {
    let mut m = setup_rehearse();
    apply(&mut m, Msg::MockExam(MockExamMsg::OpenSlides));
    apply(
        &mut m,
        Msg::MockExam(MockExamMsg::Slides(Ok(vec![SlideChoice {
            path: "slides/week1.pdf".to_owned(),
            title: "Week 1 Slides".to_owned(),
        }]))),
    );
    let mut h = harness(m);
    h.run_steps(4);
    h.get_by_label("Week 1 Slides").click();
    h.run_steps(4);
    assert!(matches!(
        &h.state().0.state.modal,
        Some(Modal::MockExamFormat(modal))
            if matches!(&modal.source, ExamSource::Slide(p) if p == "slides/week1.pdf")
    ));
    h.get_by_label("Generate exam").click();
    h.run_steps(4);
    assert!(h.state().1.iter().any(|e| matches!(
        e,
        Effect::MockExam(MockExamEffect::Generate {
            source: ExamSource::Slide(p),
            ..
        }) if p == "slides/week1.pdf"
    )));
}
