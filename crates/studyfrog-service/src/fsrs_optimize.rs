//! FSRS parameter evaluation metrics and optimization runner.
//!
//! Evaluates calibration quality (RMSE, log-loss) and fits personal FSRS-6 weights
//! directly from historical review logs in SQLite.

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::memory::MemState;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::uid::Uid;
use studyfrog_srs::fsrs::next_state;
use studyfrog_srs::optimizer::{
    AdamConfig, ReviewRecord, default_adam_config, optimize_fsrs_parameters, predict_retrievability,
};
use studyfrog_srs::params::{Params, params_default};
use studyfrog_store::error::StoreError;
use studyfrog_store::reviews::{TimedReview, load_all_reviews_timed};

use crate::schedule::{calibrated_initial, timing_difficulty};

/// First-answer difficulties a calibrated start learns from at most (as the store reads them).
const MAX_OBSERVED: usize = 500;

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

/// Replay an item's chronological reviews into optimization sample records, from the plain prior.
pub fn reviews_to_records(reviews: &[Review], params: &Params) -> Vec<ReviewRecord> {
    reviews_to_records_calibrated(reviews, params, &[])
}

/// Like [`reviews_to_records`], but the first state starts from the calibrated prior: its difficulty
/// is blended with `observed` (see `schedule::calibrated_initial`). With no observations it is the same.
pub fn reviews_to_records_calibrated(
    reviews: &[Review],
    params: &Params,
    observed: &[f64],
) -> Vec<ReviewRecord> {
    let mut out = Vec::new();
    let mut state: Option<MemState> = None;
    for r in reviews {
        match state {
            None => {
                state = Some(calibrated_initial(r.rating, params, observed));
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

/// The difficulty of each item's first timed answer, with the instant it was made, oldest first.
fn first_observations(all: &HashMap<Uid, Vec<TimedReview>>) -> Vec<(i64, f64)> {
    let mut out: Vec<(i64, f64)> = all
        .values()
        .filter_map(|reviews| {
            let (review, timing) = reviews.iter().min_by_key(|(r, _)| r.at.0)?;
            let &(answer, expected) = timing.as_ref()?;
            Some((review.at.0, timing_difficulty(answer, expected)))
        })
        .collect();
    out.sort_by_key(|p| p.0);
    out
}

/// The difficulties of the first answers made before `at`: the newest [`MAX_OBSERVED`].
fn observed_before(observations: &[(i64, f64)], at: i64) -> Vec<f64> {
    let end = observations.partition_point(|p| p.0 < at);
    observations[end.saturating_sub(MAX_OBSERVED)..end]
        .iter()
        .map(|p| p.1)
        .collect()
}

/// Load historical reviews from SQLite and convert them into FSRS optimization samples using custom
/// params. With `calibrated`, each item's first state starts from the calibrated prior (STORY-0177).
pub fn load_review_records_calibrated(
    conn: &Connection,
    params: &Params,
    calibrated: bool,
) -> Result<Vec<ReviewRecord>, StoreError> {
    let all = load_all_reviews_timed(conn)?;
    let observations = if calibrated {
        first_observations(&all)
    } else {
        Vec::new()
    };
    let mut items: Vec<_> = all.into_iter().collect();
    items.sort_by_key(|(uid, _)| *uid);
    let mut records = Vec::new();
    for (_uid, mut timed_reviews) in items {
        timed_reviews.sort_by_key(|(r, _)| r.at.0);
        let observed = timed_reviews
            .first()
            .map_or_else(Vec::new, |(r, _)| observed_before(&observations, r.at.0));
        let reviews: Vec<Review> = timed_reviews.into_iter().map(|(r, _)| r).collect();
        records.extend(reviews_to_records_calibrated(&reviews, params, &observed));
    }
    Ok(records)
}

/// Load historical reviews from SQLite and convert them into FSRS optimization samples using custom params.
pub fn load_review_records_with_params(
    conn: &Connection,
    params: &Params,
) -> Result<Vec<ReviewRecord>, StoreError> {
    load_review_records_calibrated(conn, params, false)
}

/// Load historical reviews from SQLite and convert them into FSRS optimization samples using default params.
pub fn load_review_records(conn: &Connection) -> Result<Vec<ReviewRecord>, StoreError> {
    load_review_records_with_params(conn, &params_default())
}
