//! Difficulty updates.

use studyfrog_core::review::{Rating, rating_value};

use crate::params::Params;

/// Difficulty bounds.
pub const MIN_DIFFICULTY: f32 = 1.0;
/// Difficulty bounds.
pub const MAX_DIFFICULTY: f32 = 10.0;

fn clamp_difficulty(d: f32) -> f32 {
    d.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}

/// `D0(G) = w4 - exp(w5 * (G - 1)) + 1`, before clamping.
fn initial_difficulty_raw(rating: Rating, p: &Params) -> f32 {
    let g = f32::from(rating_value(rating));
    p.weights[4] - (p.weights[5] * (g - 1.0)).exp() + 1.0
}

/// Difficulty after the first review.
pub fn initial_difficulty(rating: Rating, p: &Params) -> f32 {
    clamp_difficulty(initial_difficulty_raw(rating, p))
}

/// Difficulty after a later review: linear damping, then mean reversion towards `D0(Easy)`.
pub fn next_difficulty(difficulty: f32, rating: Rating, p: &Params) -> f32 {
    let g = f32::from(rating_value(rating));
    let delta = -p.weights[6] * (g - 3.0);
    let damped = difficulty + (10.0 - difficulty) * delta / 9.0;
    let target = initial_difficulty_raw(Rating::Easy, p);
    clamp_difficulty(p.weights[7] * target + (1.0 - p.weights[7]) * damped)
}
