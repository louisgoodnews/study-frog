//! The data of the true-retention and forgetting-curve report (STORY-0107). Plain data.

/// Reviews counted together: how many, how many were recalled, and the sum of what FSRS predicted.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RateCount {
    /// Reviews.
    pub reviews: u32,
    /// Of those, not rated Again.
    pub recalled: u32,
    /// Sum of the predicted recall probability `R(t, S)` of these reviews.
    pub predicted_sum: f32,
    /// Sum of the relative elapsed time `t / S` of these reviews.
    pub ratio_sum: f32,
}

/// The measured recall share, `None` when empty.
pub fn rate_of(c: &RateCount) -> Option<f32> {
    (c.reviews > 0).then(|| c.recalled as f32 / c.reviews as f32)
}

/// The mean predicted recall, `None` when empty.
pub fn predicted_of(c: &RateCount) -> Option<f32> {
    (c.reviews > 0).then(|| c.predicted_sum / c.reviews as f32)
}

/// Reviews whose relative elapsed time `t / S` falls in `[from_ratio, to_ratio)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurveBin {
    /// Lower edge of `t / S`.
    pub from_ratio: f32,
    /// Upper edge (`f32::INFINITY` for the last bin).
    pub to_ratio: f32,
    /// What was measured and predicted.
    pub count: RateCount,
}

/// The forgetting curve of one stability tier: items whose stability at review time was in
/// `[from_days, to_days)`.
#[derive(Debug, Clone, PartialEq)]
pub struct TierCurve {
    /// Lowest stability in the tier, in days.
    pub from_days: f32,
    /// Upper edge (`f32::INFINITY` for the last tier).
    pub to_days: f32,
    /// One bin per range of `t / S`.
    pub bins: Vec<CurveBin>,
    /// All reviews of the tier.
    pub total: RateCount,
    /// Root mean squared difference between measured and predicted recall over the bins, weighted
    /// by their reviews; `None` without reviews.
    pub rmse: Option<f32>,
}

/// True retention, retention by maturity and timeliness, and the forgetting curves by tier.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SurvivalReport {
    /// Mature items (`S >= 21` days) reviewed on time (`0.8 <= t / I <= 1.2`).
    pub true_retention: RateCount,
    /// Items with `S < 21` days, any timing.
    pub young: RateCount,
    /// Items with `S >= 21` days, any timing.
    pub mature: RateCount,
    /// Reviews before 80 % of the interval.
    pub early: RateCount,
    /// Reviews within 20 % of the interval.
    pub on_time: RateCount,
    /// Reviews after 120 % of the interval.
    pub overdue: RateCount,
    /// The four stability tiers.
    pub tiers: Vec<TierCurve>,
    /// Factor by which the learner's memory lasts longer (>1) or shorter (<1) than FSRS predicts,
    /// fitted on the curves; `None` with too few reviews.
    pub memory_scale: Option<f32>,
    /// Days until a typical item (median stability now) falls to 50 % recall, by the model.
    pub model_half_life_days: Option<f32>,
    /// The same with the fitted memory scale: the learner's own half-life.
    pub half_life_days: Option<f32>,
}
