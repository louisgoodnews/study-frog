#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the psychometric formulas: known values, round trips, and invalid inputs.

use studyfrog_rehearsal::psychometric::{
    confidence_interval, reliability_from_se, spearman_brown_prophecy,
    standard_error_of_measurement, standard_score_transform, z_for_level,
};

const TOL: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < TOL
}

#[test]
fn standard_error_matches_the_textbook_value() {
    // s = 15, r = 0.91 gives 15 * sqrt(0.09) = 4.5.
    assert!(close(standard_error_of_measurement(15.0, 0.91), 4.5));
    assert!(close(standard_error_of_measurement(10.0, 1.0), 0.0));
    assert!(standard_error_of_measurement(10.0, 1.2).is_nan());
}

#[test]
fn reliability_round_trips_through_the_standard_error() {
    for r in [0.65, 0.7, 0.8, 0.9, 0.98] {
        let se = standard_error_of_measurement(12.0, r);
        assert!(close(reliability_from_se(12.0, se), r), "r = {r}");
    }
}

#[test]
fn confidence_interval_is_symmetric_about_the_mean() {
    let (lo, hi) = confidence_interval(100.0, 1.96, 4.5);
    assert!(close(lo, 100.0 - 1.96 * 4.5));
    assert!(close(hi, 100.0 + 1.96 * 4.5));
    assert!(close((lo + hi) / 2.0, 100.0));
}

#[test]
fn standard_scores_convert_and_invert_without_drift() {
    // z = (130 - 100) / 15 = 2, so IQ = 100 + 15 * 2 = 130 and T = 50 + 10 * 2 = 70.
    assert!(close(
        standard_score_transform(130.0, 100.0, 15.0, 100.0, 15.0),
        130.0
    ));
    assert!(close(
        standard_score_transform(130.0, 100.0, 15.0, 50.0, 10.0),
        70.0
    ));
    let t = standard_score_transform(47.3, 52.0, 11.0, 50.0, 10.0);
    let raw = 52.0 + 11.0 * (t - 50.0) / 10.0;
    assert!(close(raw, 47.3));
    assert!(standard_score_transform(1.0, 0.0, 0.0, 0.0, 1.0).is_nan());
}

#[test]
fn spearman_brown_doubles_a_test_and_keeps_its_fixed_points() {
    assert!(close(spearman_brown_prophecy(0.6, 2.0), 0.75));
    assert!(close(spearman_brown_prophecy(0.6, 1.0), 0.6));
    assert!(close(spearman_brown_prophecy(0.0, 3.0), 0.0));
    assert!(spearman_brown_prophecy(0.6, 0.0).is_nan());
}

#[test]
fn z_values_follow_the_table_for_one_and_two_tails() {
    assert_eq!(z_for_level(0.95, true), Some(1.96));
    assert_eq!(z_for_level(0.95, false), Some(1.645));
    assert_eq!(z_for_level(0.42, true), None);
}
