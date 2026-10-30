#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Retention, latency, forecast and its backtest.

use proptest::prelude::*;
use studyfrog_core::event::{Difficulty, EventKind, RunEvent};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_report::backtest::{backtest, compare, estimate_model};
use studyfrog_report::forecast::{Model, ScheduledItem, forecast, model_default};
use studyfrog_report::latency::{FlipSample, flip_samples, histogram, latency_stats, percentile};
use studyfrog_report::retention::{ReviewPoint, retention_curve, retention_rate};
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_srs::params::params_default;

fn point(rating: Rating, elapsed_days: f32) -> ReviewPoint {
    ReviewPoint {
        rating,
        elapsed_days,
    }
}

#[test]
fn retention_is_counted_per_gap_and_ignores_same_day_reviews() {
    let points = [
        point(Rating::Good, 1.0),
        point(Rating::Again, 1.0),
        point(Rating::Easy, 3.2),
        point(Rating::Good, 4.0),
        point(Rating::Hard, 200.0),
        point(Rating::Good, 0.2), // the same day: not a spaced review
    ];
    let curve = retention_curve(&points);
    let at = |from: u32| curve.iter().find(|b| b.from_days == from).unwrap();
    assert_eq!((at(1).reviews, at(1).recalled), (2, 1));
    assert_eq!(retention_rate(at(1)), Some(0.5));
    assert_eq!(
        (at(3).reviews, at(3).recalled),
        (2, 2),
        "3 and 4 days share a bucket"
    );
    assert_eq!((at(91).reviews, at(91).to_days), (1, u32::MAX));
    assert_eq!(retention_rate(at(8)), None, "an empty bucket has no rate");
    assert_eq!(curve.iter().map(|b| b.reviews).sum::<u32>(), 5);
}

#[test]
fn percentiles_use_the_nearest_rank_and_the_histogram_bins_every_sample() {
    let mut ms: Vec<u32> = (1..=100).map(|i| i * 100).collect(); // 100 ms ... 10 s
    let s = latency_stats(&mut ms).unwrap();
    assert_eq!(
        (s.n, s.p50, s.p90, s.p95, s.max),
        (100, 5_000, 9_000, 9_500, 10_000)
    );
    assert_eq!(percentile(&[7], 95), Some(7));
    assert_eq!(percentile(&[], 50), None);
    assert_eq!(latency_stats(&mut []), None);
    let bins = histogram(&ms);
    assert_eq!(bins.len(), 9);
    assert_eq!(bins.iter().sum::<u32>(), 100);
    assert_eq!(bins[0], 9, "under one second: 100 ms to 900 ms");
}

fn ev(run: u128, seq: u32, t: u32, item: Option<u128>, kind: EventKind) -> RunEvent {
    RunEvent {
        run: Uid(run),
        seq,
        t_ms: t,
        item: item.map(Uid),
        kind,
    }
}

#[test]
fn flip_times_exclude_pauses_and_carry_the_mark_whenever_it_was_given() {
    let events = [
        ev(1, 0, 0, None, EventKind::RunStarted),
        ev(1, 1, 0, Some(1), EventKind::ItemShown),
        ev(
            1,
            2,
            1_000,
            Some(1),
            EventKind::ItemMarked(Difficulty::Difficult),
        ), // before the flip
        ev(1, 3, 2_000, None, EventKind::RunPaused),
        ev(1, 4, 7_000, None, EventKind::RunResumed),
        ev(1, 5, 9_000, Some(1), EventKind::ItemFlipped),
        ev(
            1,
            6,
            9_500,
            Some(1),
            EventKind::ItemMarked(Difficulty::Medium),
        ), // after: the last one wins
        ev(1, 7, 10_000, Some(1), EventKind::ItemRated(Rating::Good)),
        ev(1, 8, 10_000, Some(2), EventKind::ItemShown),
        ev(1, 9, 10_400, Some(2), EventKind::ItemSkipped),
        ev(2, 0, 0, Some(3), EventKind::ItemShown),
        ev(2, 1, 1_500, Some(3), EventKind::AnswerSubmitted("x".into())),
    ];
    assert_eq!(
        flip_samples(&events),
        [
            FlipSample {
                item: Uid(1),
                mark: Some(Difficulty::Medium),
                ms: 4_000
            },
            FlipSample {
                item: Uid(3),
                mark: None,
                ms: 1_500
            },
        ]
    );
}

#[test]
fn a_projection_follows_the_intervals_and_adds_lapses_and_new_items() {
    let m = Model {
        growth: 2.0,
        first_interval_days: 2.0,
        lapse_rate: 0.0,
        new_per_day: 0.0,
    };
    let due_today = [ScheduledItem {
        due_day: 10,
        interval_days: 1.0,
    }];
    let f = forecast(&due_today, &m, 10, 12);
    let busy: Vec<usize> = f
        .iter()
        .enumerate()
        .filter(|(_, x)| **x > 0.0)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(busy, [0, 2, 6], "today, then gaps of 2 and 4 days");
    let overdue = [ScheduledItem {
        due_day: 3,
        interval_days: 1.0,
    }];
    assert_eq!(
        forecast(&overdue, &m, 10, 3)[0],
        1.0,
        "overdue items count today"
    );

    let lapses = Model {
        lapse_rate: 0.5,
        ..m
    };
    assert_eq!(forecast(&due_today, &lapses, 10, 3), [1.0, 0.5, 1.0]);

    let fresh = Model {
        new_per_day: 2.0,
        ..m
    };
    let f = forecast(&[], &fresh, 0, 6);
    assert_eq!(
        f,
        [2.0, 2.0, 4.0, 4.0, 4.0, 4.0],
        "new items come back after 2, then 4 days"
    );
    assert_eq!(forecast(&[], &model_default(), 0, 5), [0.0; 5]);
}

#[test]
fn comparing_a_forecast_with_what_happened() {
    let same = compare(&[1.0, 2.0, 3.0], &[1, 2, 3]);
    assert_eq!(
        (same.mae_per_day, same.weekly_error, same.bias),
        (0.0, 0.0, 0.0)
    );
    let over = compare(&[2.0; 14], &[1; 14]);
    assert_eq!((over.days, over.mae_per_day), (14, 1.0));
    assert!((over.weekly_error - 1.0).abs() < 1e-6 && (over.bias - 1.0).abs() < 1e-6);
    assert_eq!(compare(&[], &[]).days, 0);
}

#[test]
fn the_simulated_learner_is_repeatable_and_behaves_like_spaced_repetition() {
    let c = SimConfig {
        seed: 7,
        days: 120,
        new_per_day: 10,
        miss_percent: 10,
    };
    let a = simulate_history(&c, &params_default());
    assert_eq!(
        a,
        simulate_history(&c, &params_default()),
        "same seed, same history"
    );
    assert_ne!(
        a,
        simulate_history(&SimConfig { seed: 8, ..c }, &params_default())
    );
    let new = a.iter().filter(|r| r.day == 50).count();
    assert!(
        new > 10,
        "day 50 has the 10 new items plus due reviews: {new}"
    );
    let again = a.iter().filter(|r| r.rating == Rating::Again).count() as f32 / a.len() as f32;
    assert!((0.02..0.3).contains(&again), "lapse share {again}");
    let model = estimate_model(&a, 90);
    assert!((model.new_per_day - 10.0).abs() < 0.5, "{model:?}");
    assert!(model.growth > 1.2 && model.growth <= 5.0, "{model:?}");
}

#[test]
fn the_forecast_is_close_on_a_simulated_history() {
    let params = params_default();
    for seed in [1, 2, 3] {
        let c = SimConfig {
            seed,
            days: 150,
            new_per_day: 12,
            miss_percent: 10,
        };
        let history = simulate_history(&c, &params);
        let e = backtest(&history, 119, 28, &params).unwrap();
        assert!(e.weekly_error < 0.35, "seed {seed}: {e:?}");
        assert!(e.bias.abs() < 0.3, "seed {seed}: {e:?}");
    }
    assert!(
        backtest(&[], 10, 7, &params).is_none(),
        "no history, no backtest"
    );
}

proptest! {
    #[test]
    fn forecasts_are_never_negative_and_have_the_requested_length(
        due in prop::collection::vec((-20_i64..40, 1.0_f32..200.0), 0..40),
        growth in 1.0_f32..4.0, lapse in 0.0_f32..0.5, new in 0.0_f32..30.0, days in 1_usize..60,
    ) {
        let items: Vec<ScheduledItem> = due.iter().map(|(d, i)| ScheduledItem { due_day: *d, interval_days: *i }).collect();
        let m = Model { growth, first_interval_days: 2.0, lapse_rate: lapse, new_per_day: new };
        let f = forecast(&items, &m, 0, days);
        prop_assert_eq!(f.len(), days);
        prop_assert!(f.iter().all(|x| *x >= 0.0 && x.is_finite()));
        let again = forecast(&items, &m, 0, days);
        prop_assert_eq!(f, again);
    }
}
