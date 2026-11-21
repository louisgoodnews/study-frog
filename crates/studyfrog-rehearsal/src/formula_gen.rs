//! Seeded synthetic exercises (STORY-0182): plausible reliabilities, standard deviations, means and
//! raw scores, so the same seed always gives the same exercise. Pure.

use crate::rng::{splitmix_next, uniform};

/// One generated exercise.
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaExercise {
    /// Standard deviation of the test.
    pub s_x: f64,
    /// Reliability of the test.
    pub r_xx: f64,
    /// Mean of the test.
    pub mean: f64,
    /// A person's raw score.
    pub raw: f64,
}

fn round_to(x: f64, places: i32) -> f64 {
    let f = 10f64.powi(places);
    (x * f).round() / f
}

/// An exercise from a seed. Reliability is in `[0.65, 0.98]`, the standard deviation in `[10, 15]`,
/// the mean in `[40, 60]` (whole numbers), and the raw score within two standard deviations of it.
pub fn generate_formula_exercise(seed: u64) -> FormulaExercise {
    let mut state = seed;
    let _ = splitmix_next(&mut state);
    let r_xx = round_to(uniform(&mut state, 0.65, 0.98), 2);
    let s_x = round_to(uniform(&mut state, 10.0, 15.0), 1);
    let mean = uniform(&mut state, 40.0, 60.0).round();
    let raw = round_to(mean + s_x * uniform(&mut state, -2.0, 2.0), 1);
    FormulaExercise {
        s_x,
        r_xx,
        mean,
        raw,
    }
}
