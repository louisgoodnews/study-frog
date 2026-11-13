#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! UI tests for the Self-assessment and Metacognitive Calibration report.

mod common;

use common::reports::bundle;
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::civil::Date;
use studyfrog_core::review::Rating;
use studyfrog_report::calibration::{
    CalibrationReport, calibration_report, calibration_review, diary_point,
};
use studyfrog_ui::explorer_ui::ReportTab;
use studyfrog_ui::state::Screen;

fn sample_calibration_report() -> CalibrationReport {
    let d1 = Date {
        year: 2026,
        month: 11,
        day: 3,
    };
    let d2 = Date {
        year: 2026,
        month: 11,
        day: 4,
    };
    let entries = vec![
        diary_point(d1, &[("performance", 3), ("focus", 4)]),
        diary_point(d1, &[("understanding", 4)]),
        diary_point(d2, &[("performance", 3), ("confidence", 4)]),
    ];
    let reviews = vec![
        calibration_review(d1, Rating::Good, Some(0.70)),
        calibration_review(d1, Rating::Good, Some(0.75)),
        calibration_review(d1, Rating::Good, Some(0.75)),
        calibration_review(d1, Rating::Again, Some(0.65)),
        calibration_review(d2, Rating::Good, Some(0.80)),
        calibration_review(d2, Rating::Good, Some(0.85)),
        calibration_review(d2, Rating::Good, Some(0.80)),
        calibration_review(d2, Rating::Again, Some(0.80)),
    ];
    calibration_report(&entries, &reviews)
}

#[test]
fn calibration_tab_renders_metrics_over_time_and_entry_count() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Calibration;

    let mut b = bundle();
    b.calibration = sample_calibration_report();
    m.state.reports = Some(b);

    let mut h = harness(m);
    h.run_steps(4);

    // Verify sample size indicator says how many entries it uses
    assert!(
        h.query_by_label_contains("Based on 3 diary entries across 8 reviews")
            .is_some(),
        "missing entry count indicator"
    );

    // Verify metric cards
    assert!(
        h.query_by_label("Self-rated performance").is_some(),
        "missing self-rated performance card"
    );
    assert!(
        h.query_by_label("Measured accuracy").is_some(),
        "missing measured accuracy card"
    );
    assert!(
        h.query_by_label("Average familiarity").is_some(),
        "missing average familiarity card"
    );
    assert!(
        h.query_by_label("Calibration score").is_some(),
        "missing calibration score card"
    );

    // Verify assessment
    assert!(
        h.query_by_label("Accurately calibrated").is_some(),
        "missing assessment label"
    );

    // Verify over-time timeline section
    assert!(
        h.query_by_label("Performance over time").is_some(),
        "missing timeline heading"
    );
    assert!(
        h.query_by_label_contains("2026-11-04").is_some(),
        "missing date bucket 2026-11-04"
    );
    assert!(
        h.query_by_label_contains("2026-11-03").is_some(),
        "missing date bucket 2026-11-03"
    );

    // Accessibility check: every control has an accessible name
    assert_all_named(&h, "calibration report");
}

#[test]
fn clicking_self_assessment_tab_navigates_to_calibration_view() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Performance;

    let mut b = bundle();
    b.calibration = sample_calibration_report();
    m.state.reports = Some(b);

    let mut h = harness(m);
    h.run_steps(2);

    h.get_by_label("Self-assessment").click();
    h.run_steps(2);

    assert!(h.query_by_label("Metacognitive calibration").is_some());
}

#[test]
fn empty_diary_history_shows_friendly_guidance() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Calibration;

    let mut b = bundle();
    b.calibration = calibration_report(&[], &[]);
    m.state.reports = Some(b);

    let mut h = harness(m);
    h.run_steps(2);

    assert!(
        h.query_by_label_contains("Based on 0 diary entries across 0 reviews")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("No diary entries with self-assessment ratings yet")
            .is_some()
    );
    assert_all_named(&h, "empty calibration report");
}
