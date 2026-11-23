#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The curves of the cognitive profile (BUG-0030): velocity over study time, answer-time bins and
//! the semantic accuracy trend.

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_service::cognitive_curves::{BINS, latency_bins, semantic_trend, velocity_curve};
use studyfrog_store::reports::HistoryRow;
use studyfrog_store::telemetry::TelemetryPoint;

fn row(rating: Rating) -> HistoryRow {
    HistoryRow {
        item: Uid(1),
        rating,
        at_ms: 0,
        elapsed_days: 0.0,
        answer_ms: None,
        expected_ms: None,
    }
}

fn point(semantic: Option<f32>) -> TelemetryPoint {
    TelemetryPoint {
        item: Uid(1),
        reviewed_at_ms: 0,
        latency_ms: 0,
        dwell_ms: 0,
        semantic_score: semantic,
        text_complexity: 0,
    }
}

#[test]
fn velocity_curve_ends_at_the_overall_rate() {
    let (a, b) = (row(Rating::Good), row(Rating::Again));
    let curve = velocity_curve(&[(&a, 60_000, 1), (&b, 60_000, 1)]);
    assert_eq!(curve.len(), 2);
    assert!((curve[0].0 - 1.0).abs() < 1e-6 && (curve[0].1 - 1.0).abs() < 1e-6);
    assert!((curve[1].0 - 2.0).abs() < 1e-6 && (curve[1].1 - 0.5).abs() < 1e-6);
}

#[test]
fn latency_bins_count_each_answer_once_and_cap_the_last_bin() {
    let r = row(Rating::Good);
    let bins = latency_bins(&[(&r, 500, 1), (&r, 2_500, 1), (&r, 999_000, 1)]);
    assert_eq!(bins.len(), BINS);
    assert_eq!((bins[0], bins[1], bins[BINS - 1]), (1, 1, 1));
    assert_eq!(bins.iter().sum::<u32>(), 3);
}

#[test]
fn semantic_trend_is_a_running_mean_of_the_scored_answers() {
    let trend = semantic_trend(&[point(Some(1.0)), point(None), point(Some(0.5))]);
    assert_eq!(trend.len(), 2);
    assert!((trend[1].0 - 2.0).abs() < 1e-6);
    assert!((trend[1].1 - 0.75).abs() < 1e-6);
    assert!(semantic_trend(&[point(None)]).is_empty());
}
