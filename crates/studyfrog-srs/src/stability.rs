//! Stability updates.

use studyfrog_core::review::{Rating, rating_value};

use crate::params::Params;

/// Lower bound for stability, in days.
pub const STABILITY_MIN: f32 = 0.001;

/// Stability after the first review: `S0(G) = w[G - 1]`.
pub fn initial_stability(rating: Rating, p: &Params) -> f32 {
    p.weights[usize::from(rating_value(rating)) - 1].max(STABILITY_MIN)
}

/// Same-day review (less than one day since the last): `S * e^(w17 (G - 3 + w18)) * S^-w19`.
/// Recall ratings never shrink stability here.
pub fn short_term_stability(s: f32, rating: Rating, p: &Params) -> f32 {
    let g = f32::from(rating_value(rating));
    let increase = (p.weights[17] * (g - 3.0 + p.weights[18])).exp() * s.powf(-p.weights[19]);
    let increase = if rating == Rating::Again {
        increase
    } else {
        increase.max(1.0)
    };
    (s * increase).max(STABILITY_MIN)
}

/// Successful recall (Hard, Good or Easy) after at least one day.
fn recall_stability(d: f32, s: f32, r: f32, rating: Rating, p: &Params) -> f32 {
    let w = &p.weights;
    let hard_penalty = if rating == Rating::Hard { w[15] } else { 1.0 };
    let easy_bonus = if rating == Rating::Easy { w[16] } else { 1.0 };
    let gain = w[8].exp()
        * (11.0 - d)
        * s.powf(-w[9])
        * (((1.0 - r) * w[10]).exp() - 1.0)
        * hard_penalty
        * easy_bonus;
    s * (1.0 + gain)
}

/// Lapse (Again) after at least one day: the smaller of the long-term and short-term results.
fn forget_stability(d: f32, s: f32, r: f32, p: &Params) -> f32 {
    let w = &p.weights;
    let long_term =
        w[11] * d.powf(-w[12]) * ((s + 1.0).powf(w[13]) - 1.0) * ((1.0 - r) * w[14]).exp();
    let short_term = s / (w[17] * w[18]).exp();
    long_term.min(short_term)
}

/// Stability after a review at least one day after the previous one.
pub fn next_stability(d: f32, s: f32, r: f32, rating: Rating, p: &Params) -> f32 {
    let next = match rating {
        Rating::Again => forget_stability(d, s, r, p),
        _ => recall_stability(d, s, r, rating, p),
    };
    next.max(STABILITY_MIN)
}
