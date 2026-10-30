#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! SM-2 tests: a hand-computed sequence from the original rules, plus properties.

use proptest::prelude::*;
use studyfrog_core::review::Rating::{self, Again, Easy, Good, Hard};
use studyfrog_srs::sm2::{MIN_EASE, sm2_initial, sm2_next};

const MAX: u32 = 36500;

fn run(ratings: &[Rating]) -> Vec<(u32, f32)> {
    let mut s = sm2_initial();
    ratings
        .iter()
        .map(|r| {
            s = sm2_next(&s, *r, MAX);
            (s.interval_days, s.ease)
        })
        .collect()
}

#[test]
fn all_good_follows_one_six_then_ceil_times_ease() {
    // q = 4 leaves EF at 2.5: 1, 6, ceil(6 * 2.5) = 15, ceil(15 * 2.5) = 38, ceil(38 * 2.5) = 95.
    let days: Vec<u32> = run(&[Good; 5]).iter().map(|x| x.0).collect();
    assert_eq!(days, [1, 6, 15, 38, 95]);
}

#[test]
fn failure_restarts_without_changing_ease() {
    let seq = run(&[Good, Good, Good, Again, Good, Good]);
    assert_eq!(seq[3], (1, seq[2].1));
    assert_eq!([seq[4].0, seq[5].0], [1, 6]);
}

#[test]
fn hard_lowers_and_easy_raises_ease() {
    assert!((run(&[Hard])[0].1 - 2.36).abs() < 1e-5);
    assert!((run(&[Easy])[0].1 - 2.6).abs() < 1e-5);
}

fn rating() -> impl Strategy<Value = Rating> {
    prop_oneof![Just(Again), Just(Hard), Just(Good), Just(Easy)]
}

proptest! {
    #[test]
    fn ease_and_interval_stay_in_bounds(ratings in prop::collection::vec(rating(), 0..200)) {
        for (days, ease) in run(&ratings) {
            prop_assert!(ease >= MIN_EASE);
            prop_assert!((1..=MAX).contains(&days));
        }
    }

    #[test]
    fn again_always_resets(ratings in prop::collection::vec(rating(), 0..50)) {
        let mut s = sm2_initial();
        for r in ratings { s = sm2_next(&s, r, MAX); }
        let after = sm2_next(&s, Again, MAX);
        prop_assert_eq!((after.repetitions, after.interval_days, after.ease), (0, 1, s.ease));
    }
}
