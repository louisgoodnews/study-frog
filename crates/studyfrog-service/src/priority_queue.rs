//! Priority-based queue ordering and session budget slicing.
//!
//! Uses `studyfrog_srs::priority_slice` to rank candidate items by blending user priority,
//! retrievability shortfall `(1 - R)`, and familiarity shortfall `(1 - F)` into an effective
//! priority score, then slicing to the requested budget.

use studyfrog_core::priority::Priority;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::candidate::{Candidate, is_due, is_reviewed};
use studyfrog_srs::fsrs::retrievability;
use studyfrog_srs::params::{Params, params_default};
use studyfrog_srs::priority_slice::{
    PriorityWeights, calculate_effective_priority, default_priority_weights, slice_by_budget,
};

const MS_PER_DAY: f64 = 86_400_000.0;

/// Maps a categorical user priority to a normalized `f64` in `[0.0, 1.0]`.
pub fn priority_to_f64(priority: Priority) -> f64 {
    match priority {
        Priority::Lowest => 0.0,
        Priority::Low => 0.25,
        Priority::Medium => 0.50,
        Priority::High => 0.75,
        Priority::Highest => 1.0,
    }
}

/// Computes the retrievability `R(t)` of a candidate item at time `now`.
///
/// If FSRS memory state is present, calculates retrievability based on stability and elapsed time.
/// If the candidate has never been reviewed, defaults to 0.9.
/// If reviewed without memory state, retrievability drops as overdue/elapsed time increases,
/// so urgency `(1 - R)` is proportional to elapsed time.
pub fn candidate_retrievability(candidate: &Candidate, now: UnixMs, params: &Params) -> f64 {
    if let Some(mem) = candidate.memory {
        let elapsed_days = (now.0.saturating_sub(candidate.last_review.unwrap_or(now).0)) as f32
            / MS_PER_DAY as f32;
        retrievability(&mem, elapsed_days, params) as f64
    } else if !is_reviewed(candidate) {
        0.9
    } else {
        let elapsed_ms = if let Some(due) = candidate.due {
            now.0.saturating_sub(due.0)
        } else if let Some(last_review) = candidate.last_review {
            now.0.saturating_sub(last_review.0)
        } else {
            0
        };
        let elapsed_days = elapsed_ms as f64 / MS_PER_DAY;
        (0.9 / (1.0 + elapsed_days)).clamp(0.0, 1.0)
    }
}

/// Normalizes candidate familiarity in `[0.0, 1.0]`, defaulting to 0.0 when missing.
pub fn candidate_familiarity(candidate: &Candidate) -> f64 {
    candidate.familiarity.unwrap_or(0.0) as f64
}

/// Computes effective priority score for a candidate in `[0.0, 1.0]`.
pub fn candidate_effective_priority(
    candidate: &Candidate,
    weights: &PriorityWeights,
    now: UnixMs,
    params: &Params,
) -> f64 {
    let p = priority_to_f64(candidate.priority);
    let r = candidate_retrievability(candidate, now, params);
    let f = candidate_familiarity(candidate);
    calculate_effective_priority(p, r, f, weights)
}

/// True if candidate is due or has never been reviewed.
pub fn is_due_or_new(candidate: &Candidate, now: UnixMs) -> bool {
    !is_reviewed(candidate) || is_due(candidate, now)
}

/// Builds a priority-ordered queue of item Uids sliced to `budget`.
///
/// Filters candidates that are due or new, sorts descending by effective priority score,
/// and slices to `budget` items.
pub fn build_priority_queue(
    candidates: &[Candidate],
    budget: usize,
    weights: Option<&PriorityWeights>,
    now: UnixMs,
) -> Vec<Uid> {
    if budget == 0 {
        return Vec::new();
    }
    let default_weights = default_priority_weights();
    let weights = weights.unwrap_or(&default_weights);
    let params = params_default();

    let admitted: Vec<Candidate> = candidates
        .iter()
        .copied()
        .filter(|c| is_due_or_new(c, now))
        .collect();

    let sliced = slice_by_budget(&admitted, budget, |c| {
        candidate_effective_priority(c, weights, now, &params)
    });

    sliced.into_iter().map(|c| c.uid).collect()
}
