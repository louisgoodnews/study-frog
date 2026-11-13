//! Continuous priority queue and anti-paralysis scoring (`docs/v0.9.0-roadmap.md`).
//!
//! Blends normalized user priority, retrievability shortfall `(1 - R)`, and familiarity shortfall `(1 - F)`
//! into an effective priority score in `[0.0, 1.0]`, allowing high-yield review queue ranking and slicing.

/// Weights used to combine user priority, retrievability, and familiarity
/// into an effective priority score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriorityWeights {
    /// Weight assigned to user-specified priority in `[0.0, 1.0]`.
    pub priority_weight: f64,
    /// Weight assigned to retrievability shortfall `1.0 - R(t)`.
    pub retrievability_weight: f64,
    /// Weight assigned to familiarity shortfall `1.0 - F(c)`.
    pub familiarity_weight: f64,
}

/// Returns the default weights for effective priority scoring:
/// 40% priority, 35% retrievability, 25% familiarity.
pub fn default_priority_weights() -> PriorityWeights {
    PriorityWeights {
        priority_weight: 0.40,
        retrievability_weight: 0.35,
        familiarity_weight: 0.25,
    }
}

impl Default for PriorityWeights {
    fn default() -> Self {
        default_priority_weights()
    }
}

/// Calculates the effective priority for an item combining priority, retrievability, and familiarity.
///
/// Formula:
/// `EffectivePriority = w_p * priority + w_r * (1.0 - retrievability) + w_f * (1.0 - familiarity)`
///
/// - `priority`: normalized user priority in `[0.0, 1.0]` (0.0 = lowest, 1.0 = highest)
/// - `retrievability`: estimated retention probability `R(t)` in `[0.0, 1.0]` (lower retention means higher urgency: `1 - R(t)`)
/// - `familiarity`: maturity/familiarity score `F(c)` in `[0.0, 1.0]` (lower familiarity means newer card / higher urgency: `1 - F(c)`)
///
/// All inputs and final score are clamped to `[0.0, 1.0]`.
pub fn calculate_effective_priority(
    priority: f64,
    retrievability: f64,
    familiarity: f64,
    weights: &PriorityWeights,
) -> f64 {
    let p = if priority.is_nan() {
        0.0
    } else {
        priority.clamp(0.0, 1.0)
    };
    let r = if retrievability.is_nan() {
        0.0
    } else {
        retrievability.clamp(0.0, 1.0)
    };
    let f = if familiarity.is_nan() {
        0.0
    } else {
        familiarity.clamp(0.0, 1.0)
    };

    let raw_score = weights.priority_weight * p
        + weights.retrievability_weight * (1.0 - r)
        + weights.familiarity_weight * (1.0 - f);

    if raw_score.is_nan() {
        0.0
    } else {
        raw_score.clamp(0.0, 1.0)
    }
}

/// Sorts a slice of items in descending order of their effective priority score.
///
/// Items with higher priority scores appear first in the slice.
/// Ties preserve their relative order (stable sort).
pub fn rank_by_priority<T, F>(items: &mut [T], score_fn: F)
where
    F: Fn(&T) -> f64,
{
    items.sort_by(|a, b| {
        let score_a = score_fn(a);
        let score_b = score_fn(b);
        score_b.total_cmp(&score_a)
    });
}

/// Slices an item collection by scoring, ranking descending, and truncating to `budget`.
pub fn slice_by_budget<T: Clone, F>(items: &[T], budget: usize, score_fn: F) -> Vec<T>
where
    F: Fn(&T) -> f64,
{
    let mut ranked = items.to_vec();
    rank_by_priority(&mut ranked, score_fn);
    ranked.truncate(budget);
    ranked
}
