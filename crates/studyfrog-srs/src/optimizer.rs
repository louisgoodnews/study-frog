//! Adam gradient descent optimizer for FSRS parameters.
//!
//! Fits FSRS parameters to historical review logs by minimizing binary
//! cross-entropy log-loss with L2 shrinkage toward published default priors.

use crate::params::DEFAULT_WEIGHTS;
use crate::stability::STABILITY_MIN;

/// Clamping lower bounds for each of the 21 FSRS parameters.
pub const PARAM_LOWER_BOUNDS: [f32; 21] = [
    0.01, 0.01, 0.01, 0.01, 1.0, 0.01, 0.01, 0.0, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.01,
];

/// Clamping upper bounds for each of the 21 FSRS parameters.
pub const PARAM_UPPER_BOUNDS: [f32; 21] = [
    100.0, 100.0, 100.0, 100.0, 10.0, 5.0, 5.0, 1.0, 5.0, 1.0, 5.0, 5.0, 1.0, 1.0, 5.0, 1.0, 10.0,
    5.0, 5.0, 1.0, 0.8,
];

/// Historical review log record used for calibration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReviewRecord {
    /// Card stability before the review event.
    pub stability: f32,
    /// Card difficulty before the review event.
    pub difficulty: f32,
    /// Days elapsed since the preceding review.
    pub elapsed_days: f32,
    /// Whether the user successfully recalled the item.
    pub recalled: bool,
}

/// Constructor for [`ReviewRecord`].
pub fn review_record(
    stability: f32,
    difficulty: f32,
    elapsed_days: f32,
    recalled: bool,
) -> ReviewRecord {
    ReviewRecord {
        stability,
        difficulty,
        elapsed_days,
        recalled,
    }
}

/// Build [`ReviewRecord`] from numeric rating (1 = Again, >1 = recalled).
pub fn review_record_from_rating(
    stability: f32,
    difficulty: f32,
    elapsed_days: f32,
    rating: u8,
) -> ReviewRecord {
    ReviewRecord {
        stability,
        difficulty,
        elapsed_days,
        recalled: rating > 1,
    }
}

/// Configuration for the Adam gradient descent optimizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdamConfig {
    /// Step size scaling factor.
    pub learning_rate: f32,
    /// Exponential decay rate for first moment estimates.
    pub beta1: f32,
    /// Exponential decay rate for second moment estimates.
    pub beta2: f32,
    /// Numerical stabilizer preventing division by zero.
    pub epsilon: f32,
    /// Maximum optimization epochs to run.
    pub max_epochs: usize,
    /// Regularization coefficient pulling weights toward published defaults.
    pub l2_weight: f32,
}

/// Published default configuration for Adam optimization.
pub fn default_adam_config() -> AdamConfig {
    AdamConfig {
        learning_rate: 0.01,
        beta1: 0.9,
        beta2: 0.999,
        epsilon: 1e-8,
        max_epochs: 50,
        l2_weight: 0.001,
    }
}

/// Results produced by parameter optimization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptimizationResult {
    /// Optimized 21-parameter weight vector.
    pub optimized_weights: [f32; 21],
    /// Objective loss before optimization.
    pub initial_loss: f32,
    /// Objective loss after optimization.
    pub final_loss: f32,
    /// Number of epochs executed.
    pub epochs_run: usize,
}

/// Clamp weights into valid physical parameter intervals.
pub fn clamp_weights(w: &mut [f32; 21]) {
    for i in 0..21 {
        if w[i].is_nan() || w[i].is_infinite() {
            w[i] = DEFAULT_WEIGHTS[i];
        } else {
            w[i] = w[i].clamp(PARAM_LOWER_BOUNDS[i], PARAM_UPPER_BOUNDS[i]);
        }
    }
}

/// Probability of recall predicted by FSRS-6 forgetting curve.
pub fn predict_retrievability(weights: &[f32; 21], elapsed_days: f32, stability: f32) -> f32 {
    let s = stability.max(STABILITY_MIN);
    let t = elapsed_days.max(0.0);
    let decay = -weights[20].clamp(PARAM_LOWER_BOUNDS[20], PARAM_UPPER_BOUNDS[20]);
    let factor = 0.9_f32.powf(1.0 / decay) - 1.0;
    (1.0 + factor * (t / s)).powf(decay)
}

/// Binary cross-entropy log-loss across review records with L2 regularization.
pub fn binary_cross_entropy_loss(
    weights: &[f32; 21],
    samples: &[ReviewRecord],
    l2_reg: f32,
) -> f32 {
    let bce = if samples.is_empty() {
        0.0
    } else {
        let sum: f32 = samples
            .iter()
            .map(|record| {
                let r = predict_retrievability(weights, record.elapsed_days, record.stability)
                    .clamp(1e-7, 1.0 - 1e-7);
                let y = if record.recalled { 1.0_f32 } else { 0.0_f32 };
                -(y * r.ln() + (1.0 - y) * (1.0 - r).ln())
            })
            .sum();
        sum / (samples.len() as f32)
    };

    let mut l2_sum = 0.0_f32;
    for i in 0..21 {
        let diff = weights[i] - DEFAULT_WEIGHTS[i];
        l2_sum += diff * diff;
    }

    bce + l2_reg * l2_sum
}

/// Numerical gradient estimation using central finite differences.
pub fn compute_numerical_gradients(
    weights: &[f32; 21],
    samples: &[ReviewRecord],
    l2_reg: f32,
    h: f32,
) -> [f32; 21] {
    let mut grads = [0.0_f32; 21];
    let mut perturbed = *weights;
    for i in 0..21 {
        let orig = weights[i];
        perturbed[i] = orig + h;
        let loss_pos = binary_cross_entropy_loss(&perturbed, samples, l2_reg);
        perturbed[i] = orig - h;
        let loss_neg = binary_cross_entropy_loss(&perturbed, samples, l2_reg);
        perturbed[i] = orig;
        grads[i] = (loss_pos - loss_neg) / (2.0 * h);
    }
    grads
}

/// Calibrate FSRS weights to historical review samples using Adam gradient descent.
pub fn optimize_fsrs_parameters(
    samples: &[ReviewRecord],
    initial_weights: Option<[f32; 21]>,
    config: &AdamConfig,
) -> OptimizationResult {
    let mut weights = initial_weights.unwrap_or(DEFAULT_WEIGHTS);
    clamp_weights(&mut weights);

    let initial_loss = binary_cross_entropy_loss(&weights, samples, config.l2_weight);
    let mut m = [0.0_f32; 21];
    let mut v = [0.0_f32; 21];
    let h = 1e-4_f32;
    let mut epochs_run = 0;

    for epoch in 1..=config.max_epochs {
        epochs_run = epoch;
        let grads = compute_numerical_gradients(&weights, samples, config.l2_weight, h);

        let beta1_t = config.beta1.powi(epoch as i32);
        let beta2_t = config.beta2.powi(epoch as i32);
        let bc1 = (1.0 - beta1_t).max(1e-8);
        let bc2 = (1.0 - beta2_t).max(1e-8);

        for i in 0..21 {
            let g = grads[i];
            m[i] = config.beta1 * m[i] + (1.0 - config.beta1) * g;
            v[i] = config.beta2 * v[i] + (1.0 - config.beta2) * g * g;

            let m_hat = m[i] / bc1;
            let v_hat = v[i] / bc2;

            weights[i] -= config.learning_rate * m_hat / (v_hat.sqrt() + config.epsilon);
        }

        clamp_weights(&mut weights);
    }

    let final_loss = binary_cross_entropy_loss(&weights, samples, config.l2_weight);

    OptimizationResult {
        optimized_weights: weights,
        initial_loss,
        final_loss,
        epochs_run,
    }
}
