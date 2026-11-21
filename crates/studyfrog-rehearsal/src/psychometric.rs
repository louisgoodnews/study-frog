//! Psychometric formulas (STORY-0182): standard error of measurement, confidence intervals, standard
//! score conversions, the Spearman-Brown prophecy, and the usual z-values. Pure functions.

/// Standard error of measurement: `s_x * sqrt(1 - r_xx)`. `s_x` is the test's standard deviation and
/// `r_xx` its reliability. Reliability outside `[0, 1]` gives `NaN`.
pub fn standard_error_of_measurement(s_x: f64, r_xx: f64) -> f64 {
    if !(0.0..=1.0).contains(&r_xx) {
        return f64::NAN;
    }
    s_x * (1.0 - r_xx).sqrt()
}

/// The reliability implied by a standard error: the inverse of [`standard_error_of_measurement`].
pub fn reliability_from_se(s_x: f64, se: f64) -> f64 {
    1.0 - (se / s_x).powi(2)
}

/// Confidence interval `(mean - z*se, mean + z*se)`.
pub fn confidence_interval(mean: f64, z: f64, se: f64) -> (f64, f64) {
    let half = z * se;
    (mean - half, mean + half)
}

/// Convert a raw score to a standard score with the given target mean and standard deviation, for
/// example `T = 50 + 10z` or `IQ = 100 + 15z`. `sd` must be positive, otherwise `NaN`.
pub fn standard_score_transform(
    raw: f64,
    mean: f64,
    sd: f64,
    target_mean: f64,
    target_sd: f64,
) -> f64 {
    if sd <= 0.0 {
        return f64::NAN;
    }
    target_mean + target_sd * (raw - mean) / sd
}

/// The Spearman-Brown prophecy: the reliability of a test lengthened by factor `k`.
/// `k * r / (1 + (k - 1) * r)`. `k` must be positive, otherwise `NaN`.
pub fn spearman_brown_prophecy(r_old: f64, k: f64) -> f64 {
    if k <= 0.0 {
        return f64::NAN;
    }
    k * r_old / (1.0 + (k - 1.0) * r_old)
}

/// The two-sided or one-sided z-value for a confidence level such as `0.95`, from the usual table.
/// `None` for levels not in the table.
pub fn z_for_level(level: f64, two_tailed: bool) -> Option<f64> {
    let table: [(f64, f64, f64); 3] = [
        (0.90, 1.645, 1.282),
        (0.95, 1.960, 1.645),
        (0.99, 2.576, 2.326),
    ];
    table
        .iter()
        .find(|(l, _, _)| (l - level).abs() < 1e-9)
        .map(|&(_, two, one)| if two_tailed { two } else { one })
}
