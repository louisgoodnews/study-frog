//! Tests for Adam optimizer calibration of FSRS parameters.

use studyfrog_srs::optimizer::{
    PARAM_LOWER_BOUNDS, PARAM_UPPER_BOUNDS, binary_cross_entropy_loss, clamp_weights,
    default_adam_config, optimize_fsrs_parameters, predict_retrievability, review_record,
    review_record_from_rating,
};
use studyfrog_srs::params::DEFAULT_WEIGHTS;

#[test]
fn default_config_values() {
    let cfg = default_adam_config();
    assert_eq!(cfg.learning_rate, 0.01);
    assert_eq!(cfg.beta1, 0.9);
    assert_eq!(cfg.beta2, 0.999);
    assert_eq!(cfg.epsilon, 1e-8);
    assert_eq!(cfg.max_epochs, 50);
    assert_eq!(cfg.l2_weight, 0.001);
}

#[test]
fn review_record_construction() {
    let r1 = review_record(5.0, 3.0, 2.0, true);
    assert_eq!(r1.stability, 5.0);
    assert_eq!(r1.difficulty, 3.0);
    assert_eq!(r1.elapsed_days, 2.0);
    assert!(r1.recalled);

    let r_again = review_record_from_rating(4.0, 2.5, 1.0, 1);
    assert!(!r_again.recalled);

    let r_hard = review_record_from_rating(4.0, 2.5, 1.0, 2);
    assert!(r_hard.recalled);

    let r_good = review_record_from_rating(4.0, 2.5, 1.0, 3);
    assert!(r_good.recalled);

    let r_easy = review_record_from_rating(4.0, 2.5, 1.0, 4);
    assert!(r_easy.recalled);
}

#[test]
fn retrievability_prediction() {
    let w = DEFAULT_WEIGHTS;
    let r_zero = predict_retrievability(&w, 0.0, 10.0);
    assert!((r_zero - 1.0).abs() < 1e-4);

    let r_at_stability = predict_retrievability(&w, 10.0, 10.0);
    assert!((r_at_stability - 0.9).abs() < 1e-4);

    let r_after = predict_retrievability(&w, 50.0, 10.0);
    assert!(r_after < r_at_stability);
    assert!(r_after > 0.0);
}

#[test]
fn loss_on_empty_samples() {
    let loss_default = binary_cross_entropy_loss(&DEFAULT_WEIGHTS, &[], 0.001);
    assert_eq!(loss_default, 0.0);

    let mut perturbed = DEFAULT_WEIGHTS;
    perturbed[0] += 2.0;
    let loss_perturbed = binary_cross_entropy_loss(&perturbed, &[], 0.001);
    assert!((loss_perturbed - (0.001 * 4.0)).abs() < 1e-5);
}

#[test]
fn optimizer_pulls_perturbed_weights_to_defaults_without_samples() {
    let mut perturbed = DEFAULT_WEIGHTS;
    perturbed[20] = 0.50; // default is 0.1542
    let mut cfg = default_adam_config();
    cfg.max_epochs = 80;
    cfg.learning_rate = 0.05;

    let res = optimize_fsrs_parameters(&[], Some(perturbed), &cfg);
    assert_eq!(res.epochs_run, 80);
    assert!(res.final_loss < res.initial_loss);
    // Perturbed weight should move closer to default
    assert!(
        (res.optimized_weights[20] - DEFAULT_WEIGHTS[20]).abs()
            < (perturbed[20] - DEFAULT_WEIGHTS[20]).abs()
    );
}

#[test]
fn optimizer_reduces_loss_on_synthetic_reviews() {
    // Generate synthetic reviews based on DEFAULT_WEIGHTS
    let mut samples = Vec::new();
    for i in 1..=100 {
        let elapsed = (i as f32) * 0.2;
        let stability = 5.0;
        let p_recall = predict_retrievability(&DEFAULT_WEIGHTS, elapsed, stability);
        let recalled = p_recall >= 0.75;
        samples.push(review_record(stability, 5.0, elapsed, recalled));
    }

    // Start with perturbed initial weights
    let mut initial = DEFAULT_WEIGHTS;
    initial[20] = 0.50;

    let mut cfg = default_adam_config();
    cfg.learning_rate = 0.02;
    cfg.max_epochs = 50;

    let res = optimize_fsrs_parameters(&samples, Some(initial), &cfg);
    assert_eq!(res.epochs_run, 50);
    assert!(
        res.final_loss < res.initial_loss,
        "final_loss {} should be less than initial_loss {}",
        res.final_loss,
        res.initial_loss
    );
    // Weight should move from 0.50 closer to optimal DEFAULT_WEIGHTS[20] (~0.1542)
    assert!(
        (res.optimized_weights[20] - DEFAULT_WEIGHTS[20]).abs()
            < (initial[20] - DEFAULT_WEIGHTS[20]).abs()
    );
}

#[test]
fn parameter_clamping_and_invariants() {
    let mut invalid = [-100.0_f32; 21];
    invalid[0] = f32::NAN;
    invalid[1] = f32::INFINITY;
    clamp_weights(&mut invalid);

    assert_eq!(invalid[0], DEFAULT_WEIGHTS[0]);
    assert_eq!(invalid[1], DEFAULT_WEIGHTS[1]);

    for i in 2..21 {
        assert!(invalid[i] >= PARAM_LOWER_BOUNDS[i]);
        assert!(invalid[i] <= PARAM_UPPER_BOUNDS[i]);
    }
}
