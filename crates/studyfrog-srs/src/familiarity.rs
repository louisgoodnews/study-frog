//! How familiarity tempers a scheduled interval (`docs/research/familiarity-and-fsrs.md`, model M2).
//!
//! The factor `m = exp(beta * (F - F_expected(S)))` compares the familiarity `F` with the familiarity a card of
//! stability `S` usually has. It is 1 when they agree, below 1 when the user knows the item less well than FSRS
//! assumes, above 1 when better. The stored memory state is never changed: only the interval put on the calendar.

use studyfrog_core::memory::MemState;

use crate::fsrs::interval_exact;
use crate::params::Params;

/// The constants of the factor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FactorParams {
    /// How strongly familiarity moves the interval; 0 switches it off.
    pub strength: f32,
    /// Stability (days) at which the expected familiarity is one half.
    pub s_half: f32,
    /// Smallest and largest factor.
    pub bounds: (f32, f32),
    /// Reviews after which the strength is fully applied; it ramps in from the second to this one.
    pub full_at_review: u32,
}

/// The defaults of the research: strength 2, half at 30 days, factor between 0.25 and 2, nothing before the third review.
pub fn factor_params_default() -> FactorParams {
    FactorParams {
        strength: 2.0,
        s_half: 30.0,
        bounds: (0.25, 2.0),
        full_at_review: 4,
    }
}

/// The familiarity a card of this stability usually has: `S / (S + s_half)`.
pub fn expected_familiarity(stability: f32, p: &FactorParams) -> f32 {
    let s = stability.max(0.0);
    s / (s + p.s_half)
}

/// The share of the strength applied after `reviews` reviews: none up to the second, all from `full_at_review`.
fn ramp(reviews: u32, p: &FactorParams) -> f32 {
    let from = 2_u32;
    let span = p.full_at_review.saturating_sub(from).max(1) as f32;
    (reviews.saturating_sub(from) as f32 / span).clamp(0.0, 1.0)
}

/// The factor for an item with familiarity `fam` and stability `stability` after `reviews` reviews.
pub fn familiarity_factor(fam: f32, stability: f32, reviews: u32, p: &FactorParams) -> f32 {
    let beta = p.strength * ramp(reviews, p);
    if beta == 0.0 {
        return 1.0;
    }
    let gap = fam.clamp(0.0, 1.0) - expected_familiarity(stability, p);
    (beta * gap).exp().clamp(p.bounds.0, p.bounds.1)
}

/// The FSRS interval multiplied by `factor`, rounded, at least one day and at most the maximum.
pub fn interval_days_with(state: &MemState, factor: f32, params: &Params) -> u32 {
    let days = (interval_exact(state, params) * factor).round().max(1.0);
    (days as u32).min(params.maximum_interval_days)
}

/// An SM-2 interval (its state keeps the unscaled one) multiplied by `factor`.
pub fn scaled_interval_days(base_days: u32, factor: f32, maximum_days: u32) -> u32 {
    let days = (base_days as f32 * factor).round().max(1.0);
    (days as u32).min(maximum_days)
}

/// The interval as chosen by familiarity, kept from running past a deadline: a longer interval than the base one is
/// cut back to the days left until an exam or due date, but never below the base interval itself.
pub fn before_deadline(days: u32, base_days: u32, days_to_deadline: Option<u32>) -> u32 {
    match days_to_deadline {
        Some(left) if days > base_days => days.min(left.max(base_days)).max(1),
        _ => days,
    }
}
