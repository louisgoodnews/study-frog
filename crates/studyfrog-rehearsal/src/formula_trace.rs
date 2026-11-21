//! Step-by-step trace of the standard error of measurement, and a diagnosis of a student's working
//! (STORY-0183). Pure: the student's intermediate values are compared with the correct trace, and
//! the first step that deviates is named with the most likely mistake.

use crate::psychometric::standard_error_of_measurement;

/// One operation of the trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOp {
    /// `1 - r_xx`.
    OneMinusReliability,
    /// The square root of the previous value.
    SquareRoot,
    /// Multiply by the standard deviation `s_x`.
    MultiplyBySd,
}

/// One step with the value it produces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraceStep {
    /// The operation.
    pub op: StepOp,
    /// The value after the operation.
    pub value: f64,
}

/// The typical mistakes the diagnosis can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mistake {
    /// The square root was left out.
    ForgotRoot,
    /// The variance `s_x^2` was used where the standard deviation belongs.
    UsedVariance,
    /// The subtraction was done the wrong way round: `r_xx - 1`.
    InvertedSubtraction,
    /// The value is close to correct but rounded too early.
    Rounding,
    /// The value matches no known mistake.
    Unexplained,
}

/// The result of checking a student's working.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnosis {
    /// Index of the first step that deviates, or `None` when all submitted steps are right.
    pub deviates_at: Option<usize>,
    /// The most likely mistake at that step.
    pub mistake: Option<Mistake>,
    /// A sentence the student can read.
    pub message: String,
}

/// Relative tolerance for a value to count as correct.
const CORRECT_TOL: f64 = 0.01;
/// Relative tolerance inside which a deviation is blamed on rounding.
const ROUNDING_TOL: f64 = 0.05;

/// The three steps of the standard error of measurement, with their values.
pub fn se_trace(s_x: f64, r_xx: f64) -> Vec<TraceStep> {
    let one_minus = 1.0 - r_xx;
    let root = one_minus.sqrt();
    vec![
        TraceStep {
            op: StepOp::OneMinusReliability,
            value: one_minus,
        },
        TraceStep {
            op: StepOp::SquareRoot,
            value: root,
        },
        TraceStep {
            op: StepOp::MultiplyBySd,
            value: standard_error_of_measurement(s_x, r_xx),
        },
    ]
}

fn close(submitted: f64, expected: f64, tol: f64) -> bool {
    (submitted - expected).abs() <= tol * expected.abs().max(1e-12)
}

/// The mistake behind a wrong value at step `index`, judged against the other candidate forms.
fn classify(index: usize, submitted: f64, s_x: f64, r_xx: f64) -> Mistake {
    let one_minus = 1.0 - r_xx;
    let root = one_minus.sqrt();
    let expected = [one_minus, root, s_x * root][index.min(2)];
    let candidates: [(f64, Mistake); 3] = match index {
        0 => [
            (r_xx - 1.0, Mistake::InvertedSubtraction),
            (f64::NAN, Mistake::Unexplained),
            (f64::NAN, Mistake::Unexplained),
        ],
        1 => [
            (one_minus, Mistake::ForgotRoot),
            (f64::NAN, Mistake::Unexplained),
            (f64::NAN, Mistake::Unexplained),
        ],
        _ => [
            (s_x * one_minus, Mistake::ForgotRoot),
            (s_x * s_x * root, Mistake::UsedVariance),
            (f64::NAN, Mistake::Unexplained),
        ],
    };
    for (value, mistake) in candidates {
        if value.is_finite() && close(submitted, value, CORRECT_TOL) {
            return mistake;
        }
    }
    if close(submitted, expected, ROUNDING_TOL) {
        Mistake::Rounding
    } else {
        Mistake::Unexplained
    }
}

/// The sentence the student reads for a mistake.
pub fn mistake_message(m: Mistake) -> &'static str {
    match m {
        Mistake::ForgotRoot => {
            "You left out the square root. The standard error needs sqrt(1 - r), not 1 - r."
        }
        Mistake::UsedVariance => {
            "You used the variance (s squared). The standard error multiplies the standard deviation s, not s squared."
        }
        Mistake::InvertedSubtraction => {
            "Subtract the reliability from 1, not the other way round: 1 - r."
        }
        Mistake::Rounding => {
            "This is close: check the rounding. Keep more decimals until the last step."
        }
        Mistake::Unexplained => {
            "This step does not match the formula. Go back to the step before and check it."
        }
    }
}

/// Check the student's submitted intermediate values, in trace order, against the correct trace.
/// Steps after the first deviation are not judged, since they depend on it.
pub fn diagnose_se(s_x: f64, r_xx: f64, submitted: &[f64]) -> Diagnosis {
    let trace = se_trace(s_x, r_xx);
    for (i, (step, &value)) in trace.iter().zip(submitted).enumerate() {
        if !close(value, step.value, CORRECT_TOL) {
            let mistake = classify(i, value, s_x, r_xx);
            return Diagnosis {
                deviates_at: Some(i),
                mistake: Some(mistake),
                message: mistake_message(mistake).to_owned(),
            };
        }
    }
    Diagnosis {
        deviates_at: None,
        mistake: None,
        message: "All steps are correct.".to_owned(),
    }
}
