//! Calibration between subjective diary ratings and measured review performance.
//! See `docs/specs/diary.md` Section 7.

use std::collections::BTreeSet;

use studyfrog_core::review::Rating;

pub use crate::calibration_types::*;

fn day_self_score(entries: &[&DiaryRatingPoint]) -> (Option<f32>, Vec<f32>) {
    let scores: Vec<f32> = entries
        .iter()
        .filter_map(|e| entry_score(&e.ratings))
        .collect();
    if scores.is_empty() {
        (None, Vec::new())
    } else {
        let s: f32 = scores.iter().sum();
        (Some(s / scores.len() as f32), scores)
    }
}

/// Calculated recall accuracy for an individual review based on rating, item familiarity/novelty,
/// and reading speed latency.
pub fn review_accuracy(r: &CalibrationReview) -> f32 {
    let rating_weight = match r.rating {
        Rating::Again => 0.0,
        Rating::Hard => 0.60,
        Rating::Good => 0.85,
        Rating::Easy => 1.00,
    };
    if rating_weight == 0.0 {
        return 0.0;
    }
    // Items that have been viewed repeatedly and matured have proven stability.
    // Rather than heavily penalizing new items (which crushed recall accuracy to ~20%),
    // familiarity acts as a subtle confidence weight (0.85 for brand new, 1.00 for fully mature).
    let fam = r.familiarity.unwrap_or(0.2).clamp(0.0, 1.0);
    let fam_weight = 0.85 + 0.15 * fam;

    // View latency vs expected reading speed: prolonged hesitation indicates struggling retrieval.
    let latency_weight = match (r.answer_ms, r.expected_ms) {
        (Some(ans), Some(exp)) if exp > 0 => {
            let ratio = exp as f32 / ans.max(exp) as f32;
            0.85 + 0.15 * ratio.clamp(0.0, 1.0)
        }
        _ => 1.0,
    };

    // When item difficulty is known, scale accuracy so mastering hard material is properly recognized:
    let diff_weight = match r.difficulty {
        Some(d) => {
            let norm = (d.clamp(1.0, 10.0) - 1.0) / 9.0;
            1.0 + 0.15 * norm
        }
        None => 1.0,
    };

    (rating_weight * fam_weight * latency_weight * diff_weight).clamp(0.0, 1.0)
}

fn day_review_metrics(reviews: &[&CalibrationReview]) -> (Option<f32>, Option<f32>, usize) {
    if reviews.is_empty() {
        return (None, None, 0);
    }
    let total_acc: f32 = reviews.iter().map(|r| review_accuracy(r)).sum();
    let mean_acc = total_acc / reviews.len() as f32;
    let recalled = reviews.iter().filter(|r| r.rating != Rating::Again).count();
    let fam_vals: Vec<f32> = reviews.iter().filter_map(|r| r.familiarity).collect();
    let fam = if fam_vals.is_empty() {
        None
    } else {
        let sum: f32 = fam_vals.iter().sum();
        Some(sum / fam_vals.len() as f32)
    };
    (Some(mean_acc), fam, recalled)
}

fn compute_metrics(
    daily_gaps: &[f32],
    mean_self: Option<f32>,
    mean_acc: Option<f32>,
) -> (Option<f32>, Option<f32>, CalibrationBias) {
    let delta = if !daily_gaps.is_empty() {
        let sum: f32 = daily_gaps.iter().sum();
        Some(sum / daily_gaps.len() as f32)
    } else {
        match (mean_self, mean_acc) {
            (Some(s), Some(a)) => Some(s - a),
            _ => None,
        }
    };

    let score = if !daily_gaps.is_empty() {
        let abs_err: f32 =
            daily_gaps.iter().map(|g| g.abs()).sum::<f32>() / daily_gaps.len() as f32;
        Some((1.0 - abs_err).clamp(0.0, 1.0))
    } else {
        delta.map(|d| (1.0 - d.abs()).clamp(0.0, 1.0))
    };

    let bias = match delta {
        Some(d) if d > 0.05 => CalibrationBias::Overconfident,
        Some(d) if d < -0.05 => CalibrationBias::Underconfident,
        _ => CalibrationBias::WellCalibrated,
    };

    (delta, score, bias)
}

/// Pure calculation of self-assessment calibration against measured review performance.
pub fn calibration_report(
    entries: &[DiaryRatingPoint],
    reviews: &[CalibrationReview],
) -> CalibrationReport {
    let mut dates = BTreeSet::new();
    for e in entries {
        dates.insert(e.date);
    }
    for r in reviews {
        dates.insert(r.date);
    }

    let mut buckets = Vec::with_capacity(dates.len());
    let mut entries_used = 0_usize;
    let mut all_self_scores = Vec::new();
    let mut all_fams = Vec::new();
    let mut daily_gaps = Vec::new();

    for date in dates {
        let day_entries: Vec<&DiaryRatingPoint> =
            entries.iter().filter(|e| e.date == date).collect();
        let (self_score, scores) = day_self_score(&day_entries);
        entries_used += scores.len();
        all_self_scores.extend(scores);

        let day_reviews: Vec<&CalibrationReview> =
            reviews.iter().filter(|r| r.date == date).collect();
        let (accuracy, familiarity, _recalled) = day_review_metrics(&day_reviews);
        all_fams.extend(day_reviews.iter().filter_map(|r| r.familiarity));

        let gap = match (self_score, accuracy) {
            (Some(s), Some(a)) => {
                let g = s - a;
                daily_gaps.push(g);
                Some(g)
            }
            _ => None,
        };

        buckets.push(CalibrationBucket {
            date,
            entries: day_entries.len(),
            reviews: day_reviews.len(),
            self_score,
            accuracy,
            familiarity,
            gap,
        });
    }

    let mean_self_score = (!all_self_scores.is_empty())
        .then(|| all_self_scores.iter().sum::<f32>() / all_self_scores.len() as f32);

    let total_accuracy: f32 = reviews.iter().map(review_accuracy).sum();
    let mean_accuracy = (!reviews.is_empty()).then(|| total_accuracy / reviews.len() as f32);

    let mean_familiarity =
        (!all_fams.is_empty()).then(|| all_fams.iter().sum::<f32>() / all_fams.len() as f32);

    let (delta, score, bias) = compute_metrics(&daily_gaps, mean_self_score, mean_accuracy);

    CalibrationReport {
        entries_used,
        reviews_evaluated: reviews.len(),
        mean_self_score,
        mean_accuracy,
        mean_familiarity,
        delta,
        score,
        bias,
        buckets,
    }
}
