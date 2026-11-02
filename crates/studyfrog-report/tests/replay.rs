#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The replay of a history: too few reviews, a history where familiarity is informative, invariants and the budget.

use std::time::Instant;

use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_report::familiarity::{FamContext, FamEvent};
use studyfrog_report::replay::{MIN_REVIEWS, ReplayItem, Verdict, replay};
use studyfrog_srs::params::params_default;

const DAY: i64 = 86_400_000;

/// An item rated on the given days; `speed` is expected/actual answer time of the Good ratings.
fn item(days: &[(i64, Rating)], speed: f32) -> ReplayItem {
    let ratings: Vec<(UnixMs, Rating)> = days.iter().map(|(d, r)| (UnixMs(d * DAY), *r)).collect();
    let events = ratings
        .iter()
        .map(|(at, rating)| FamEvent::Rating {
            at: *at,
            rating: *rating,
            answer_ms: Some(1000),
            expected_ms: Some((1000.0 * speed) as u32),
        })
        .collect();
    ReplayItem {
        events,
        ctx: FamContext::default(),
        ratings,
    }
}

fn informative(pairs: usize) -> Vec<ReplayItem> {
    let early = [(0, Rating::Good), (2, Rating::Good), (5, Rating::Good)];
    (0..pairs)
        .flat_map(|_| {
            let known = [early.as_slice(), &[(60, Rating::Good)]].concat();
            let unknown = [early.as_slice(), &[(60, Rating::Again)]].concat();
            [item(&known, 1.5), item(&unknown, 0.5)]
        })
        .collect()
}

#[test]
fn a_short_history_is_scored_with_the_defaults_and_gets_no_verdict() {
    let r = replay(&informative(10), &params_default());
    assert_eq!(r.scored, 60, "three scored reviews per item");
    assert_eq!(r.verdict, Verdict::TooFewReviews);
    assert_eq!((r.used.strength, r.used.s_half), (2.0, 30.0));
    assert!(replay(&[], &params_default()).bins.is_empty());
}

#[test]
fn when_familiarity_tells_who_will_recall_the_factor_helps_and_shortens_the_workload_gap() {
    let r = replay(&informative(500), &params_default());
    assert!(r.scored >= MIN_REVIEWS);
    assert_eq!(r.verdict, Verdict::FactorHelps, "{r:?}");
    assert!(r.loss.1 < r.loss.0 - 0.002, "{:?}", r.loss);
    assert!(r.used.strength > 0.0);
    assert_eq!(
        r.bins.iter().map(|b| b.reviews as usize).sum::<usize>(),
        r.scored - (r.scored as f64 * 0.8) as usize
    );
    assert!(
        r.bins
            .iter()
            .all(|b| (0.0..=1.0).contains(&b.recalled) && (0.0..=1.0).contains(&b.plain))
    );
}

#[test]
fn when_plain_fsrs_is_already_calibrated_and_familiarity_says_nothing_new_it_does_not_claim_to_help()
 {
    let same: Vec<ReplayItem> = (0..400)
        .map(|i| {
            let last = if i % 6 == 0 {
                Rating::Again
            } else {
                Rating::Good
            };
            item(
                &[
                    (0, Rating::Good),
                    (2, Rating::Good),
                    (5, Rating::Good),
                    (60, last),
                ],
                1.0,
            )
        })
        .collect();
    let r = replay(&same, &params_default());
    assert_ne!(r.verdict, Verdict::FactorHelps, "{r:?}");
}

#[test]
fn ten_thousand_reviews_replay_within_the_budget() {
    let items = informative(1700);
    let start = Instant::now();
    let r = replay(&items, &params_default());
    let took = start.elapsed();
    assert_eq!(r.scored, 10_200);
    assert!(took.as_millis() < 5000, "replaying took {took:?}");
}
