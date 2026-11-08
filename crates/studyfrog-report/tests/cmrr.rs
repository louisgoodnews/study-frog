#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Workload against desired retention and the minimum recommended retention (STORY-0106).

use studyfrog_core::memory::MemState;
use studyfrog_report::cmrr::{RETENTION_GRID, cmrr};
use studyfrog_report::sim_engine::{Pace, RatingMix, SimCard, SimMemory, SimSetup};
use studyfrog_srs::params::params_default;

fn card(stability: f32, ago: i64) -> SimCard {
    SimCard {
        memory: Some(SimMemory::Fsrs(MemState {
            stability,
            difficulty: 5.0,
        })),
        last_day: -ago,
        due_day: 0,
        relearning: false,
        sm2: false,
        pace: 0,
    }
}

fn setup(learn_s: f32) -> SimSetup {
    SimSetup {
        today: 0,
        days: 180,
        trajectories: 1,
        seed: 3,
        miss_probability: 0.0,
        new_per_day: 5.0,
        new_days: 180,
        mix: RatingMix {
            first_again: 0.2,
            hard: 0.1,
            good: 0.8,
        },
        paces: vec![Pace {
            new_s: 20.0,
            learn_s,
            review_s: 10.0,
        }],
    }
}

fn deck() -> Vec<SimCard> {
    let mut d: Vec<SimCard> = (0..600)
        .map(|i| card(1.0 + (i % 60) as f32, (i % 20) as i64))
        .collect();
    d.extend((0..600).map(|_| SimCard {
        memory: None,
        ..card(1.0, 0)
    }));
    d
}

#[test]
fn every_target_is_simulated_and_higher_targets_cost_more_reviews_at_the_top() {
    let r = cmrr(&deck(), &setup(30.0), &params_default());
    assert_eq!(r.days, 180);
    let targets: Vec<f32> = r.points.iter().map(|p| p.retention).collect();
    assert_eq!(targets, RETENTION_GRID);
    let at = |t: f32| {
        r.points
            .iter()
            .find(|p| (p.retention - t).abs() < 1e-6)
            .unwrap()
    };
    assert!(
        at(0.97).daily_reviews > at(0.90).daily_reviews * 1.4,
        "{:?} {:?}",
        at(0.97),
        at(0.90)
    );
    assert!(at(0.97).mean_retention > at(0.80).mean_retention);
    for p in &r.points {
        assert!(p.daily_minutes > 0.0 && p.memorized > 0.0);
        assert!((p.efficiency - p.memorized / p.daily_minutes).abs() < 1e-3);
    }
}

#[test]
fn the_recommendation_maximises_cards_per_minute_above_the_least_work() {
    let r = cmrr(&deck(), &setup(30.0), &params_default());
    let least = r.min_workload_retention.unwrap();
    let best = r.recommended.unwrap();
    assert!(best >= least);
    let min = r
        .points
        .iter()
        .map(|p| p.daily_minutes)
        .fold(f32::MAX, f32::min);
    assert_eq!(
        r.points
            .iter()
            .find(|p| p.daily_minutes == min)
            .unwrap()
            .retention,
        least
    );
    let top = r
        .points
        .iter()
        .filter(|p| p.retention >= least)
        .map(|p| p.efficiency)
        .fold(0.0_f32, f32::max);
    assert_eq!(
        r.points
            .iter()
            .find(|p| p.efficiency == top)
            .unwrap()
            .retention,
        best
    );
}

#[test]
fn when_forgetting_is_costly_lower_targets_add_work() {
    // Intervals capped at 10 days: a lower target cannot space reviews further out, it only adds lapses.
    let p = studyfrog_srs::params::params_new(studyfrog_srs::params::DEFAULT_WEIGHTS, 0.9, 10);
    let s = SimSetup {
        trajectories: 8,
        ..setup(300.0)
    };
    let r = cmrr(&deck(), &s, &p);
    let least = r.min_workload_retention.unwrap();
    assert!(
        (0.86..=0.96).contains(&least),
        "forgetting costs five minutes, so the cheapest target is not the lowest: {least}"
    );
    let minutes = |t: f32| {
        r.points
            .iter()
            .find(|p| p.retention == t)
            .unwrap()
            .daily_minutes
    };
    assert!(
        minutes(0.75) > minutes(least) && minutes(0.97) > minutes(least),
        "a U-shaped curve"
    );
}

#[test]
fn an_empty_deck_recommends_nothing_and_seeds_repeat() {
    let p = params_default();
    let empty = cmrr(&[], &setup(30.0), &p);
    assert_eq!(
        (empty.recommended, empty.min_workload_retention),
        (None, None)
    );
    assert_eq!(
        cmrr(&deck(), &setup(30.0), &p),
        cmrr(&deck(), &setup(30.0), &p)
    );
}
