//! FSRS-6 parameters. See `docs/specs/srs-algorithm.md` for provenance.

/// Published FSRS-6 default weights (w0..w20).
pub const DEFAULT_WEIGHTS: [f32; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796, 1.4835,
    0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];

/// Scheduler configuration plus the curve constants derived from `weights[20]`.
/// Build with [`params_new`] so the derived fields stay consistent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Params {
    /// FSRS-6 weights.
    pub weights: [f32; 21],
    /// Target retention at review time, in `(0, 1)`.
    pub request_retention: f32,
    /// Upper bound for any interval, in days.
    pub maximum_interval_days: u32,
    /// Forgetting-curve exponent, `-w20`.
    pub decay: f32,
    /// Forgetting-curve factor, `0.9^(1/decay) - 1`, so that `R(S, S) = 0.9`.
    pub factor: f32,
}

/// Build parameters and derive the curve constants.
pub fn params_new(
    weights: [f32; 21],
    request_retention: f32,
    maximum_interval_days: u32,
) -> Params {
    let decay = -weights[20];
    let factor = 0.9_f32.powf(1.0 / decay) - 1.0;
    Params {
        weights,
        request_retention,
        maximum_interval_days,
        decay,
        factor,
    }
}

/// Defaults: published weights, 90% retention, 100 years maximum.
pub fn params_default() -> Params {
    params_new(DEFAULT_WEIGHTS, 0.9, 36500)
}
