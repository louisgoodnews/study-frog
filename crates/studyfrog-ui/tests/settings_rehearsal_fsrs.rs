#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Tests for FSRS parameter optimization section on Rehearsal settings.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_service::fsrs_optimize::FsrsOptimizationMetrics;
use studyfrog_srs::params::DEFAULT_WEIGHTS;
use studyfrog_ui::state::Screen;

fn rehearsal_settings_model() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.screen = Screen::Settings;
    m.state.app.settings_tab = Some("rehearsal".to_owned());
    m
}

#[test]
fn rehearsal_settings_shows_optimize_button() {
    let mut h = harness(rehearsal_settings_model());
    h.run();
    assert_all_named(&h, "Rehearsal settings");
    assert!(h.query_by_label("Optimize FSRS Parameters").is_some());
    assert!(h.query_by_label("FSRS parameter optimization").is_some());
}

#[test]
fn rehearsal_settings_displays_metrics_when_present() {
    let mut h = harness(rehearsal_settings_model());
    h.run();

    let metrics = FsrsOptimizationMetrics {
        initial_loss: 0.4567,
        final_loss: 0.3210,
        loss_reduction_pct: 29.71,
        rmse: 0.1234,
        sample_count: 150,
        optimized_weights: DEFAULT_WEIGHTS,
    };

    studyfrog_ui::view::settings_rehearsal_fsrs::set_metrics(&h.ctx, metrics);
    h.run();

    assert!(h.query_by_label("Initial loss").is_some());
    assert!(h.query_by_label("0.4567").is_some());
    assert!(h.query_by_label("Final loss").is_some());
    assert!(h.query_by_label("0.3210").is_some());
    assert!(h.query_by_label("Improvement").is_some());
    assert!(h.query_by_label("29.71 %").is_some());
    assert!(h.query_by_label("RMSE").is_some());
    assert!(h.query_by_label("0.1234").is_some());
    assert!(h.query_by_label("Reviews evaluated").is_some());
    assert!(h.query_by_label("150").is_some());
}
