#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Tests for FSRS parameter evaluation metrics and optimization runner.

mod common;

use common::env;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::fsrs_optimize::{
    calculate_rmse, load_review_records, load_review_records_with_params, reviews_to_records,
    run_fsrs_optimization,
};
use studyfrog_srs::optimizer::{default_adam_config, review_record};
use studyfrog_srs::params::{DEFAULT_WEIGHTS, params_default};
use studyfrog_store::schedule::insert_review;

#[test]
fn rmse_on_empty_samples_is_zero() {
    assert_eq!(calculate_rmse(&DEFAULT_WEIGHTS, &[]), 0.0);
}

#[test]
fn rmse_calculation_on_synthetic_samples() {
    let samples = vec![
        review_record(10.0, 3.0, 10.0, true),
        review_record(10.0, 3.0, 10.0, false),
    ];
    let rmse = calculate_rmse(&DEFAULT_WEIGHTS, &samples);
    // Predict retrievability at t=10, S=10 is 0.9.
    // Errors are (0.9 - 1.0) = -0.1, and (0.9 - 0.0) = 0.9.
    // Squared errors: 0.01 + 0.81 = 0.82.
    // Mean: 0.41. Sqrt(0.41) ≈ 0.6403.
    assert!((rmse - 0.41_f32.sqrt()).abs() < 1e-4);
}

#[test]
fn optimization_on_empty_samples_returns_none() {
    assert!(run_fsrs_optimization(&[], None).is_none());
}

#[test]
fn optimization_reduces_loss_and_computes_metrics() {
    let mut samples = Vec::new();
    for i in 1..=80 {
        let elapsed = (i as f32) * 0.25;
        let stability = 5.0;
        let recalled = i % 3 != 0;
        samples.push(review_record(stability, 4.0, elapsed, recalled));
    }

    let mut cfg = default_adam_config();
    cfg.max_epochs = 30;

    let metrics = run_fsrs_optimization(&samples, Some(&cfg))
        .expect("optimization should produce metrics for non-empty samples");

    assert_eq!(metrics.sample_count, samples.len());
    assert!(metrics.initial_loss > 0.0);
    assert!(metrics.final_loss > 0.0);
    assert!(metrics.final_loss <= metrics.initial_loss);
    assert!(metrics.loss_reduction_pct >= 0.0);
    assert!(metrics.rmse > 0.0);
    assert_eq!(metrics.optimized_weights.len(), 21);
}

#[test]
fn reviews_to_records_converts_history_after_initial_learning() {
    let params = params_default();
    let item = Uid(11);

    // Initial review: establishes state, no prior record to evaluate.
    let r1 = Review {
        item,
        rating: Rating::Good,
        at: UnixMs(1_000_000),
        elapsed_days: 0.0,
    };
    assert!(reviews_to_records(&[r1], &params).is_empty());

    // Subsequent review: has prior stability and elapsed days.
    let r2 = Review {
        item,
        rating: Rating::Easy,
        at: UnixMs(1_086_400_000),
        elapsed_days: 1.0,
    };
    let records = reviews_to_records(&[r1, r2], &params);
    assert_eq!(records.len(), 1);
    assert!(records[0].stability > 0.0);
    assert!(records[0].difficulty > 0.0);
    assert_eq!(records[0].elapsed_days, 1.0);
    assert!(records[0].recalled);

    // Third review with Again rating (recalled = false).
    let r3 = Review {
        item,
        rating: Rating::Again,
        at: UnixMs(1_172_800_000),
        elapsed_days: 2.0,
    };
    let records3 = reviews_to_records(&[r1, r2, r3], &params);
    assert_eq!(records3.len(), 2);
    assert!(!records3[1].recalled);
    assert_eq!(records3[1].elapsed_days, 2.0);
}

#[test]
fn sqlite_reviews_integration() {
    let env = env();
    let item = Uid(11);

    let empty = load_review_records(&env.conn).unwrap();
    assert!(empty.is_empty());

    // Add first review
    insert_review(
        &env.conn,
        &Review {
            item,
            rating: Rating::Good,
            at: UnixMs(1_000_000),
            elapsed_days: 0.0,
        },
    )
    .unwrap();

    // After 1 review: 0 evaluation samples
    assert!(load_review_records(&env.conn).unwrap().is_empty());

    // Add second and third reviews
    insert_review(
        &env.conn,
        &Review {
            item,
            rating: Rating::Good,
            at: UnixMs(1_086_400_000),
            elapsed_days: 1.0,
        },
    )
    .unwrap();
    insert_review(
        &env.conn,
        &Review {
            item,
            rating: Rating::Again,
            at: UnixMs(1_172_800_000),
            elapsed_days: 3.0,
        },
    )
    .unwrap();

    let loaded = load_review_records(&env.conn).unwrap();
    assert_eq!(loaded.len(), 2);
    assert!(loaded[0].recalled);
    assert!(!loaded[1].recalled);

    let loaded_with_p = load_review_records_with_params(&env.conn, &params_default()).unwrap();
    assert_eq!(loaded_with_p.len(), 2);

    let metrics = run_fsrs_optimization(&loaded, None)
        .expect("metrics should be calculated for loaded records");
    assert_eq!(metrics.sample_count, 2);
}

#[test]
fn a_calibrated_first_state_differs_only_when_there_are_observations() {
    use studyfrog_service::fsrs_optimize::reviews_to_records_calibrated;
    let params = params_default();
    let item = Uid(12);
    let first = Review {
        item,
        rating: Rating::Good,
        at: UnixMs(1_000_000),
        elapsed_days: 0.0,
    };
    let second = Review {
        item,
        rating: Rating::Good,
        at: UnixMs(1_086_400_000),
        elapsed_days: 1.0,
    };
    let plain = reviews_to_records(&[first, second], &params);
    assert_eq!(
        reviews_to_records_calibrated(&[first, second], &params, &[]),
        plain
    );
    let slow = reviews_to_records_calibrated(&[first, second], &params, &[9.0; 20]);
    assert_eq!(slow.len(), 1);
    assert!(
        slow[0].difficulty > plain[0].difficulty,
        "{slow:?} vs {plain:?}"
    );
}
