#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI tests for WIP item-complexity warnings in the Create dialog (EPIC-077, STORY-0252).

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_report::complexity_baseline::RetentionComplexityProfile;
use studyfrog_ui::effect::{Effect, EffectImport};
use studyfrog_ui::modal::{Field, Modal, ModalMsg};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

#[test]
fn create_dialog_warns_when_wip_draft_exceeds_retained_complexity_and_can_be_dismissed() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Modal(ModalMsg::OpenCreate));
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::Import(EffectImport::LoadComplexityProfile)))
    );

    let profile = RetentionComplexityProfile {
        sample_count: 12,
        retained_avg_words: 15.0,
        retained_ceiling_words: 24.0,
    };
    let _ = apply(
        &mut m,
        Msg::Modal(ModalMsg::ComplexityProfileLoaded(Some(profile))),
    );

    let dense = "Explain all stages of cellular respiration including glycolysis, pyruvate oxidation, the citric acid cycle, oxidative phosphorylation, electron transport chain complexes I through IV, chemiosmosis, and ATP synthase mechanics in full detail.";
    let _ = apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(Field::Text, dense.into())),
    );

    let mut h = harness(m);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run_steps(4);

    assert!(
        h.query_by_label_contains("Consider splitting it into smaller cards")
            .is_some(),
        "complexity warning should be visible"
    );
    assert_all_named(&h, "create dialog with complexity warning");

    h.get_by_label("Dismiss").click();
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Consider splitting it into smaller cards")
            .is_none(),
        "dismissed warning should hide"
    );
    let Some(Modal::Create(form)) = &h.state().0.state.modal else {
        panic!("expected create modal");
    };
    assert!(form.complexity_dismissed);
}
