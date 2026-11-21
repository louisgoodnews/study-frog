#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Cognitive profile tab (STORY-0178): calibration widget, velocity and the charts' headings.

mod common;

use common::analytics::analytics;
use common::reports::bundle;
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_service::cognitive_report::profile;
use studyfrog_store::reports::HistoryRow;
use studyfrog_ui::explorer_ui::ReportTab;
use studyfrog_ui::model::Model;
use studyfrog_ui::state::Screen;

fn row(rating: Rating, at_ms: i64, answer_ms: u32, expected_ms: u32) -> HistoryRow {
    HistoryRow {
        item: Uid(1),
        rating,
        at_ms,
        elapsed_days: 1.0,
        answer_ms: Some(answer_ms),
        expected_ms: Some(expected_ms),
    }
}

fn on_cognitive(rows: &[HistoryRow]) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Cognitive;
    let mut b = bundle();
    b.analytics = Box::new(analytics());
    b.cognitive = profile(rows);
    m.state.reports = Some(b);
    m
}

#[test]
fn calibration_and_velocity_are_shown_with_their_charts() {
    let rows = [
        row(Rating::Easy, 0, 9_000, 4_000),
        row(Rating::Good, 60_000, 3_000, 4_000),
        row(Rating::Easy, 120_000, 3_000, 4_000),
    ];
    let mut h = harness(on_cognitive(&rows));
    h.run_steps(4);
    for text in [
        "Metacognitive calibration",
        "Rated Easy",
        "Answered in time",
        "Learning velocity",
        "Answer time against the length of the words",
        "Share of answers that passed, cumulative by week",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_all_named(&h, "cognitive profile");
}

#[test]
fn without_timed_answers_the_tab_says_what_is_missing() {
    let mut h = harness(on_cognitive(&[]));
    h.run_steps(4);
    assert!(
        h.query_by_label("Answer times are needed to compare your ratings with your timing.")
            .is_some()
    );
    assert!(
        h.query_by_label("Needs more timed answers to measure.")
            .is_some()
    );
}
