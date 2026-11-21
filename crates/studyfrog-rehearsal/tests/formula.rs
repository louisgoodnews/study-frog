#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the step trace, the diagnosis of student working, and the seeded generator.

use studyfrog_rehearsal::formula_gen::generate_formula_exercise;
use studyfrog_rehearsal::formula_trace::{Mistake, diagnose_se, se_trace};

// s = 10, r = 0.75: 1 - r = 0.25, sqrt = 0.5, SE = 5.
const S: f64 = 10.0;
const R: f64 = 0.75;

#[test]
fn trace_has_the_three_documented_steps() {
    let trace = se_trace(S, R);
    assert_eq!(trace.len(), 3);
    assert!((trace[0].value - 0.25).abs() < 1e-12);
    assert!((trace[1].value - 0.5).abs() < 1e-12);
    assert!((trace[2].value - 5.0).abs() < 1e-12);
}

#[test]
fn correct_working_is_accepted() {
    let d = diagnose_se(S, R, &[0.25, 0.5, 5.0]);
    assert_eq!(d.deviates_at, None);
    assert_eq!(d.mistake, None);
}

#[test]
fn forgetting_the_root_is_named_at_the_root_step() {
    let d = diagnose_se(S, R, &[0.25, 0.25]);
    assert_eq!(d.deviates_at, Some(1));
    assert_eq!(d.mistake, Some(Mistake::ForgotRoot));
}

#[test]
fn using_variance_instead_of_sd_is_named_at_the_last_step() {
    // s^2 * sqrt(1 - r) = 100 * 0.5 = 50.
    let d = diagnose_se(S, R, &[0.25, 0.5, 50.0]);
    assert_eq!(d.deviates_at, Some(2));
    assert_eq!(d.mistake, Some(Mistake::UsedVariance));
    assert!(d.message.contains("variance"));
}

#[test]
fn inverted_subtraction_is_recognised() {
    let d = diagnose_se(S, R, &[-0.25]);
    assert_eq!(d.deviates_at, Some(0));
    assert_eq!(d.mistake, Some(Mistake::InvertedSubtraction));
}

#[test]
fn a_slightly_off_value_is_blamed_on_rounding() {
    let d = diagnose_se(S, R, &[0.25, 0.5, 5.2]);
    assert_eq!(d.deviates_at, Some(2));
    assert_eq!(d.mistake, Some(Mistake::Rounding));
}

#[test]
fn an_unrelated_value_is_unexplained() {
    let d = diagnose_se(S, R, &[0.9, 0.5, 5.0]);
    assert_eq!(d.deviates_at, Some(0));
    assert_eq!(d.mistake, Some(Mistake::Unexplained));
}

#[test]
fn generated_exercises_are_deterministic_and_in_range() {
    assert_eq!(generate_formula_exercise(5), generate_formula_exercise(5));
    for seed in 0..200u64 {
        let e = generate_formula_exercise(seed);
        assert!((0.65..=0.98).contains(&e.r_xx), "r {}", e.r_xx);
        assert!((10.0..=15.0).contains(&e.s_x), "s {}", e.s_x);
        assert!((40.0..=60.0).contains(&e.mean), "mean {}", e.mean);
        assert!((e.raw - e.mean).abs() <= 2.0 * e.s_x + 0.1, "raw {}", e.raw);
    }
}
