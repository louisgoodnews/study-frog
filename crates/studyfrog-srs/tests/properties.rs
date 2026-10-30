#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Property tests for FSRS-6 (see `docs/contributing/TESTING.md`).

use proptest::prelude::*;
use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_srs::difficulty::{MAX_DIFFICULTY, MIN_DIFFICULTY};
use studyfrog_srs::fsrs::{
    initial_state, interval_days, interval_exact, next_state, retrievability,
};
use studyfrog_srs::fuzz::{fuzz_bounds, fuzz_interval};
use studyfrog_srs::params::params_default;
use studyfrog_srs::stability::STABILITY_MIN;

fn rating() -> impl Strategy<Value = Rating> {
    prop_oneof![
        Just(Rating::Again),
        Just(Rating::Hard),
        Just(Rating::Good),
        Just(Rating::Easy)
    ]
}

fn state() -> impl Strategy<Value = MemState> {
    (0.01_f32..2000.0, MIN_DIFFICULTY..=MAX_DIFFICULTY).prop_map(|(stability, difficulty)| {
        MemState {
            stability,
            difficulty,
        }
    })
}

proptest! {
    #[test]
    fn invariants_hold_over_any_review_sequence(
        first in rating(),
        steps in prop::collection::vec((rating(), 0_u32..400), 0..60),
    ) {
        let p = params_default();
        let mut s = initial_state(first, &p);
        for (r, elapsed) in steps {
            s = next_state(&s, r, elapsed as f32, &p);
            prop_assert!(s.stability >= STABILITY_MIN && s.stability.is_finite());
            prop_assert!((MIN_DIFFICULTY..=MAX_DIFFICULTY).contains(&s.difficulty));
            let days = interval_days(&s, &p);
            prop_assert!((1..=p.maximum_interval_days).contains(&days));
        }
    }

    #[test]
    fn recall_stability_is_monotonic_in_rating(s in state(), elapsed in 0_u32..1000) {
        let p = params_default();
        let stab = |r| next_state(&s, r, elapsed as f32, &p).stability;
        let all = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy].map(stab);
        prop_assert!(all.windows(2).all(|w| w[0] <= w[1]), "{all:?}");
    }

    #[test]
    fn retrievability_is_a_decreasing_probability(
        s in state(),
        a in 0.0_f32..5000.0,
        b in 0.0_f32..5000.0,
    ) {
        let p = params_default();
        let (near, far) = if a <= b { (a, b) } else { (b, a) };
        let (rn, rf) = (retrievability(&s, near, &p), retrievability(&s, far, &p));
        prop_assert!((0.0..=1.0).contains(&rn) && (0.0..=1.0).contains(&rf));
        prop_assert!(rn >= rf);
    }

    #[test]
    fn retrievability_at_stability_is_ninety_percent(s in state()) {
        let r = retrievability(&s, s.stability, &params_default());
        prop_assert!((r - 0.9).abs() < 1e-3, "{r}");
    }

    #[test]
    fn interval_grows_with_stability(a in 0.01_f32..5000.0, b in 0.01_f32..5000.0, d in 1.0_f32..10.0) {
        let p = params_default();
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let at = |stability| MemState { stability, difficulty: d };
        prop_assert!(interval_exact(&at(lo), &p) <= interval_exact(&at(hi), &p));
        prop_assert!(interval_days(&at(lo), &p) <= interval_days(&at(hi), &p));
    }

    #[test]
    fn interval_at_stability_equals_stability_for_ninety_percent(s in 0.5_f32..3000.0) {
        let p = params_default();
        let exact = interval_exact(&MemState { stability: s, difficulty: 5.0 }, &p);
        prop_assert!((exact - s).abs() / s < 1e-3, "{exact} vs {s}");
    }

    #[test]
    fn fuzz_stays_in_window_and_is_deterministic(
        interval in 0.5_f32..40000.0,
        unit in 0.0_f32..1.0,
        max in 30_u32..36500,
    ) {
        let fuzzed = fuzz_interval(interval, unit, max);
        prop_assert_eq!(fuzzed, fuzz_interval(interval, unit, max));
        prop_assert!(fuzzed >= 1 && fuzzed <= max);
        if interval >= 2.5 && (interval as u32) < max {
            let (low, high) = fuzz_bounds(interval, max);
            prop_assert!((low..=high).contains(&fuzzed));
        }
    }
}
