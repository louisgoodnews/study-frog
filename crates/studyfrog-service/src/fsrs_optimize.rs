//! FSRS parameter evaluation metrics and optimization runner.
//!
//! Evaluates calibration quality (RMSE, log-loss) and fits personal FSRS-6 weights
//! directly from historical review logs in SQLite.

use rusqlite::Connection;
use studyfrog_core::memory::MemState;
use studyfrog_core::review::{Rating, Review};
use studyfrog_srs::fsrs::{initial_state, next_state};
use studyfrog_srs::optimizer::{
    AdamConfig, ReviewRecord, default_adam_config, optimize_fsrs_parameters, predict_retrievability,
};
use studyfrog_srs::params::{Params, params_default};
use studyfrog_store::error::StoreError;
use studyfrog_store::reviews::load_all_reviews_timed;

/// Evaluation metrics produced by FSRS parameter calibration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FsrsOptimizationMetrics {
    /// Objective loss before optimization.
    pub initial_loss: f32,
    /// Objective loss after optimization.
    pub final_loss: f32,
    /// Percent reduction from initial to final loss.
    pub loss_reduction_pct: f32,
    /// Root mean squared error between recall predictions and outcomes.
    pub rmse: f32,
    /// Number of review samples evaluated.
    pub sample_count: usize,
    /// Optimized 21-parameter weight vector.
    pub optimized_weights: [f32; 21],
}

/// Compute Root Mean Squared Error (RMSE) of predicted retrievabilities against actual recall.
pub fn calculate_rmse(weights: &[f32; 21], samples: &[ReviewRecord]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples
        .iter()
        .map(|record| {
            let p = predict_retrievability(weights, record.elapsed_days, record.stability);
            let y = if record.recalled { 1.0_f32 } else { 0.0_f32 };
            let diff = p - y;
            diff * diff
        })
        .sum();
    (sum_sq / (samples.len() as f32)).sqrt()
}

/// Run Adam gradient descent optimization across review records and compute calibration metrics.
pub fn run_fsrs_optimization(
    samples: &[ReviewRecord],
    config: Option<&AdamConfig>,
) -> Option<FsrsOptimizationMetrics> {
    if samples.is_empty() {
        return None;
    }
    let default_cfg = default_adam_config();
    let cfg = config.unwrap_or(&default_cfg);
    let result = optimize_fsrs_parameters(samples, None, cfg);

    let loss_reduction_pct = if result.initial_loss > 0.0 {
        ((result.initial_loss - result.final_loss) / result.initial_loss) * 100.0
    } else {
        0.0
    };

    let rmse = calculate_rmse(&result.optimized_weights, samples);

    Some(FsrsOptimizationMetrics {
        initial_loss: result.initial_loss,
        final_loss: result.final_loss,
        loss_reduction_pct,
        rmse,
        sample_count: samples.len(),
        optimized_weights: result.optimized_weights,
    })
}

/// Replay an item's chronological reviews into optimization sample records.
pub fn reviews_to_records(reviews: &[Review], params: &Params) -> Vec<ReviewRecord> {
    let mut out = Vec::new();
    let mut state: Option<MemState> = None;
    for r in reviews {
        match state {
            None => {
                state = Some(initial_state(r.rating, params));
            }
            Some(prev) => {
                out.push(ReviewRecord {
                    stability: prev.stability,
                    difficulty: prev.difficulty,
                    elapsed_days: r.elapsed_days,
                    recalled: r.rating != Rating::Again,
                });
                state = Some(next_state(&prev, r.rating, r.elapsed_days, params));
            }
        }
    }
    out
}

/// Load historical reviews from SQLite and convert them into FSRS optimization samples using custom params.
pub fn load_review_records_with_params(
    conn: &Connection,
    params: &Params,
) -> Result<Vec<ReviewRecord>, StoreError> {
    let all = load_all_reviews_timed(conn)?;
    let mut items: Vec<_> = all.into_iter().collect();
    items.sort_by_key(|(uid, _)| *uid);
    let mut records = Vec::new();
    for (_uid, mut timed_reviews) in items {
        timed_reviews.sort_by_key(|(r, _)| r.at.0);
        let reviews: Vec<Review> = timed_reviews.into_iter().map(|(r, _)| r).collect();
        records.extend(reviews_to_records(&reviews, params));
    }
    Ok(records)
}

/// Load historical reviews from SQLite and convert them into FSRS optimization samples using default params.
pub fn load_review_records(conn: &Connection) -> Result<Vec<ReviewRecord>, StoreError> {
    load_review_records_with_params(conn, &params_default())
}
