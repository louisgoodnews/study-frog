//! FSRS-6 scheduling as pure functions. No clock, no randomness: elapsed time is an argument.

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;

use crate::difficulty::{initial_difficulty, next_difficulty};
use crate::params::Params;
use crate::stability::{initial_stability, next_stability, short_term_stability};

/// State after the very first review of an item.
pub fn initial_state(rating: Rating, p: &Params) -> MemState {
    MemState {
        stability: initial_stability(rating, p),
        difficulty: initial_difficulty(rating, p),
    }
}

/// Probability of recall `elapsed_days` after the last review: `(1 + factor * t / S)^decay`.
pub fn retrievability(state: &MemState, elapsed_days: f32, p: &Params) -> f32 {
    (1.0 + p.factor * elapsed_days.max(0.0) / state.stability).powf(p.decay)
}

/// State after a later review. `elapsed_days` is the whole number of days since the previous
/// review; below 1.0 the short-term (same-day) rules apply.
pub fn next_state(prev: &MemState, rating: Rating, elapsed_days: f32, p: &Params) -> MemState {
    let difficulty = next_difficulty(prev.difficulty, rating, p);
    let stability = if elapsed_days < 1.0 {
        short_term_stability(prev.stability, rating, p)
    } else {
        let r = retrievability(prev, elapsed_days, p);
        next_stability(prev.difficulty, prev.stability, r, rating, p)
    };
    MemState {
        stability,
        difficulty,
    }
}

/// Unrounded days until retrievability falls to `request_retention`.
pub fn interval_exact(state: &MemState, p: &Params) -> f32 {
    state.stability / p.factor * (p.request_retention.powf(1.0 / p.decay) - 1.0)
}

/// Whole-day interval: rounded, at least 1, at most `maximum_interval_days`.
pub fn interval_days(state: &MemState, p: &Params) -> u32 {
    let days = interval_exact(state, p).round().max(1.0);
    (days as u32).min(p.maximum_interval_days)
}
