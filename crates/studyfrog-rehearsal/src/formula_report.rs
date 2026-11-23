//! The result of a drill exercise (STORY-0182, EPIC-062): the z-score, T score and IQ-scale score of
//! a raw score, and the confidence band around it from the standard error of measurement. Uses the
//! functions of [`crate::psychometric`]. Pure.

use crate::psychometric::{confidence_interval, standard_score_transform, z_for_level};

/// The confidence level of the band shown with a drill result.
pub const BAND_LEVEL: f64 = 0.95;

/// The standard scores of one raw score, and its confidence band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreReport {
    /// The z-score: `(raw - mean) / s_x`.
    pub z: f64,
    /// The T score: `50 + 10z`.
    pub t_score: f64,
    /// The IQ-scale score: `100 + 15z`.
    pub iq_score: f64,
    /// The band `(raw - z*se, raw + z*se)` for the true score, at [`BAND_LEVEL`].
    pub band: (f64, f64),
}

/// The report for a raw score on a test with the given mean and standard deviation, and the standard
/// error of measurement `se`. A two-sided z-value at [`BAND_LEVEL`] gives the band.
pub fn score_report(raw: f64, mean: f64, s_x: f64, se: f64) -> ScoreReport {
    let z_two_sided = z_for_level(BAND_LEVEL, true).unwrap_or(1.96);
    ScoreReport {
        z: standard_score_transform(raw, mean, s_x, 0.0, 1.0),
        t_score: standard_score_transform(raw, mean, s_x, 50.0, 10.0),
        iq_score: standard_score_transform(raw, mean, s_x, 100.0, 15.0),
        band: confidence_interval(raw, z_two_sided, se),
    }
}
