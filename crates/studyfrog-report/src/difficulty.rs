//! Empirical item difficulty, learning velocity, and calibrated priors from cognitive telemetry
//! (STORY-0177). Pure: every function takes plain numbers and returns plain numbers.

/// Lowest difficulty on the scale.
pub const DIFFICULTY_MIN: f64 = 1.0;
/// Highest difficulty on the scale.
pub const DIFFICULTY_MAX: f64 = 10.0;
/// Typical reading time per word, in milliseconds (about 240 words a minute).
pub const EXPECTED_MS_PER_WORD: f64 = 250.0;
/// The prior counts as this many observations when it is blended with the data.
pub const PRIOR_WEIGHT: f64 = 5.0;

/// One answer's telemetry, as numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    /// Milliseconds before the answer was submitted.
    pub latency_ms: f64,
    /// Milliseconds of active reading on the question and answer.
    pub dwell_ms: f64,
    /// Words in the question and its reference answer.
    pub words: f64,
    /// SLM semantic score in `[0, 1]`, if one was produced.
    pub semantic: Option<f64>,
}

fn unit(x: f64) -> f64 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Empirical difficulty on `[1, 10]`: 40 % slow answering, 20 % reading time beyond what the text
/// needs, and 40 % semantic error (an unknown score counts as half wrong).
pub fn empirical_difficulty(o: &Observation) -> f64 {
    let latency = unit((o.latency_ms - 2_000.0) / 18_000.0);
    let expected = o.words.max(1.0) * EXPECTED_MS_PER_WORD;
    let dwell = unit((o.dwell_ms / expected - 1.0) / 3.0);
    let error = o.semantic.map_or(0.5, |s| 1.0 - unit(s));
    let mix = 0.4 * latency + 0.2 * dwell + 0.4 * error;
    DIFFICULTY_MIN + (DIFFICULTY_MAX - DIFFICULTY_MIN) * mix
}

/// Learning velocity: the least-squares slope of stability (days) against cumulative study time
/// (minutes), in days of stability gained per study minute. `None` with fewer than two points or
/// when all points share one study time.
pub fn learning_velocity(points: &[(f64, f64)]) -> Option<f64> {
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|p| p.0).sum::<f64>() / n;
    let mean_y = points.iter().map(|p| p.1).sum::<f64>() / n;
    let sxy: f64 = points.iter().map(|p| (p.0 - mean_x) * (p.1 - mean_y)).sum();
    let sxx: f64 = points.iter().map(|p| (p.0 - mean_x).powi(2)).sum();
    (sxx > f64::EPSILON).then(|| sxy / sxx)
}

/// Blend a prior difficulty with observed difficulties. The prior counts as [`PRIOR_WEIGHT`]
/// observations, so a few answers move it a little and many answers move it a lot.
pub fn calibrated_prior(prior: f64, observed: &[f64]) -> f64 {
    let finite: Vec<f64> = observed.iter().copied().filter(|v| v.is_finite()).collect();
    let sum: f64 = finite.iter().sum();
    let blended = (prior * PRIOR_WEIGHT + sum) / (PRIOR_WEIGHT + finite.len() as f64);
    blended.clamp(DIFFICULTY_MIN, DIFFICULTY_MAX)
}
