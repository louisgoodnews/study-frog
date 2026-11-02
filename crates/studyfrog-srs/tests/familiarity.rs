#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The familiarity factor on intervals: the worked values of the research and the properties.

use proptest::prelude::*;
use studyfrog_core::memory::MemState;
use studyfrog_srs::familiarity::{
    FactorParams, before_deadline, expected_familiarity, factor_params_default, familiarity_factor,
    interval_days_with, scaled_interval_days,
};
use studyfrog_srs::fsrs::interval_days;
use studyfrog_srs::params::params_default;

fn p() -> FactorParams {
    factor_params_default()
}

#[test]
fn the_factor_matches_the_table_of_the_research() {
    // beta 2, S_half 30; rows S = 2.3, 11, 46, 150, 400; columns F = 0.3, 0.5, 0.7, 0.9, 0.97 (after four reviews)
    let table = [
        (2.3, [1.58, 2.00, 2.00, 2.00, 2.00]),
        (11.0, [1.07, 1.59, 2.00, 2.00, 2.00]),
        (46.0, [0.54, 0.81, 1.21, 1.80, 2.00]),
        (150.0, [0.34, 0.51, 0.77, 1.14, 1.31]),
        (400.0, [0.28, 0.42, 0.63, 0.94, 1.08]),
    ];
    for (s, row) in table {
        for (f, want) in [0.3, 0.5, 0.7, 0.9, 0.97].into_iter().zip(row) {
            let got = familiarity_factor(f, s, 4, &p());
            assert!((got - want).abs() < 0.011, "S {s}, F {f}: {got} vs {want}");
        }
    }
}

#[test]
fn an_easy_rating_at_half_familiarity_comes_back_in_about_half_the_time() {
    // The owner's example: FSRS says about five months, familiarity says 0.5.
    let state = MemState {
        stability: 150.0,
        difficulty: 5.0,
    };
    let plain = interval_days(&state, &params_default());
    let factor = familiarity_factor(0.5, state.stability, 6, &p());
    let tempered = interval_days_with(&state, factor, &params_default());
    assert!(
        (tempered as f32 / plain as f32 - 0.51).abs() < 0.02,
        "{plain} -> {tempered}"
    );
}

#[test]
fn nothing_changes_before_the_third_review_and_the_strength_ramps_in() {
    let at = |reviews| familiarity_factor(0.0, 150.0, reviews, &p());
    assert_eq!((at(0), at(1), at(2)), (1.0, 1.0, 1.0));
    let (third, fourth) = (at(3), at(4));
    assert!(third < 1.0 && fourth < third, "{third} {fourth}");
    assert_eq!(at(4), at(40), "fully applied from the fourth");
}

#[test]
fn a_strength_of_zero_is_plain_fsrs() {
    let off = FactorParams {
        strength: 0.0,
        ..p()
    };
    let state = MemState {
        stability: 33.3,
        difficulty: 6.0,
    };
    for fam in [0.0, 0.5, 1.0] {
        let f = familiarity_factor(fam, state.stability, 9, &off);
        assert_eq!(f, 1.0);
        assert_eq!(
            interval_days_with(&state, f, &params_default()),
            interval_days(&state, &params_default())
        );
    }
}

#[test]
fn the_interval_is_at_least_a_day_and_never_above_the_maximum() {
    let tiny = MemState {
        stability: 0.01,
        difficulty: 5.0,
    };
    assert_eq!(interval_days_with(&tiny, 0.25, &params_default()), 1);
    let huge = MemState {
        stability: 1.0e9,
        difficulty: 5.0,
    };
    assert_eq!(
        interval_days_with(&huge, 2.0, &params_default()),
        params_default().maximum_interval_days
    );
    assert_eq!(scaled_interval_days(10, 0.01, 100), 1);
    assert_eq!(scaled_interval_days(100, 5.0, 150), 150);
}

#[test]
fn a_longer_interval_is_cut_at_the_deadline_but_never_below_the_base_interval() {
    assert_eq!(
        before_deadline(50, 30, Some(40)),
        40,
        "stretched interval stops at the exam"
    );
    assert_eq!(
        before_deadline(50, 30, Some(10)),
        30,
        "never shorter than the base because of the exam"
    );
    assert_eq!(
        before_deadline(20, 30, Some(10)),
        20,
        "a shortening is not touched"
    );
    assert_eq!(before_deadline(50, 30, None), 50);
    assert_eq!(before_deadline(30, 30, Some(5)), 30);
}

proptest! {
    #[test]
    fn a_higher_familiarity_never_lowers_the_factor(s in 0.1_f32..5000.0, a in 0.0_f32..1.0, b in 0.0_f32..1.0, reviews in 0_u32..30) {
        let (lo, hi) = (a.min(b), a.max(b));
        prop_assert!(familiarity_factor(lo, s, reviews, &p()) <= familiarity_factor(hi, s, reviews, &p()) + 1e-6);
    }

    #[test]
    fn the_factor_stays_inside_its_bounds(s in 0.0_f32..1.0e6, f in -1.0_f32..2.0, reviews in 0_u32..100) {
        let m = familiarity_factor(f, s, reviews, &p());
        prop_assert!((0.25..=2.0).contains(&m), "{m}");
    }

    #[test]
    fn the_factor_is_one_when_familiarity_is_what_the_stability_suggests(s in 0.1_f32..5000.0, reviews in 4_u32..40) {
        let expected = expected_familiarity(s, &p());
        prop_assert!((familiarity_factor(expected, s, reviews, &p()) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn the_interval_stays_in_range_for_any_factor(s in 0.001_f32..1.0e7, d in 1.0_f32..10.0, m in 0.25_f32..2.0) {
        let days = interval_days_with(&MemState { stability: s, difficulty: d }, m, &params_default());
        prop_assert!((1..=params_default().maximum_interval_days).contains(&days));
    }
}
