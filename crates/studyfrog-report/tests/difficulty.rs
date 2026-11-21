#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for empirical difficulty, learning velocity, and calibrated priors.

use studyfrog_report::difficulty::{
    DIFFICULTY_MAX, DIFFICULTY_MIN, Observation, calibrated_prior, empirical_difficulty,
    learning_velocity,
};

fn obs(latency: f64, dwell: f64, words: f64, semantic: Option<f64>) -> Observation {
    Observation {
        latency_ms: latency,
        dwell_ms: dwell,
        words,
        semantic,
    }
}

#[test]
fn difficulty_stays_on_the_scale_for_extreme_inputs() {
    for o in [
        obs(0.0, 0.0, 0.0, Some(1.0)),
        obs(1e9, 1e9, 1.0, Some(0.0)),
        obs(f64::NAN, f64::INFINITY, 10.0, None),
    ] {
        let d = empirical_difficulty(&o);
        assert!((DIFFICULTY_MIN..=DIFFICULTY_MAX).contains(&d), "{d}");
    }
}

#[test]
fn a_quick_correct_answer_is_easier_than_a_slow_wrong_one() {
    let easy = empirical_difficulty(&obs(1_500.0, 2_000.0, 40.0, Some(0.95)));
    let hard = empirical_difficulty(&obs(25_000.0, 30_000.0, 40.0, Some(0.1)));
    assert!(easy < 3.0, "{easy}");
    assert!(hard > 8.0, "{hard}");
}

#[test]
fn velocity_is_the_slope_of_stability_against_study_time() {
    let points = [(0.0, 1.0), (10.0, 3.0), (20.0, 5.0)];
    let v = learning_velocity(&points).unwrap();
    assert!((v - 0.2).abs() < 1e-9);
    assert!(learning_velocity(&[(5.0, 1.0)]).is_none());
    assert!(learning_velocity(&[(5.0, 1.0), (5.0, 2.0)]).is_none());
}

#[test]
fn calibrated_prior_moves_with_the_data_and_stays_on_the_scale() {
    assert!((calibrated_prior(5.0, &[]) - 5.0).abs() < 1e-9);
    let few = calibrated_prior(5.0, &[9.0]);
    let many = calibrated_prior(5.0, &vec![9.0; 50]);
    assert!(few > 5.0 && few < many);
    assert!(many <= DIFFICULTY_MAX);
    assert!(calibrated_prior(5.0, &[f64::NAN]).is_finite());
}
