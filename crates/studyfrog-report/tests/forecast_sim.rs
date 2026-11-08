#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The simulated forecast and the exam pacer (STORY-0105).

use proptest::prelude::*;
use studyfrog_core::memory::{MemState, Sm2State};
use studyfrog_report::exam_pacer::{exam_pace, pacer_input};
use studyfrog_report::forecast_sim::{load_total, simulate_forecast};
use studyfrog_report::sim_engine::{
    Pace, RatingMix, SimCard, SimMemory, SimSetup, day_total, rollout,
};
use studyfrog_srs::params::params_default;

const MIX: RatingMix = RatingMix {
    first_again: 0.0,
    hard: 0.0,
    good: 1.0,
};

fn fsrs(stability: f32, last_day: i64, due_day: i64) -> SimCard {
    SimCard {
        memory: Some(SimMemory::Fsrs(MemState {
            stability,
            difficulty: 5.0,
        })),
        last_day,
        due_day,
        relearning: false,
        sm2: false,
        pace: 0,
    }
}

fn new_card() -> SimCard {
    SimCard {
        memory: None,
        ..fsrs(1.0, 0, 0)
    }
}

fn setup(days: usize, trajectories: u32) -> SimSetup {
    SimSetup {
        today: 100,
        days,
        trajectories,
        seed: 7,
        miss_probability: 0.0,
        new_per_day: 0.0,
        new_days: days,
        mix: MIX,
        paces: vec![Pace {
            new_s: 60.0,
            learn_s: 45.0,
            review_s: 30.0,
        }],
    }
}

#[test]
fn a_due_card_is_reviewed_once_today_by_its_stage_and_pace() {
    let p = params_default();
    let mut deck = [fsrs(100.0, 0, 100), fsrs(5.0, 95, 99), new_card()];
    deck[1].relearning = true;
    let days = rollout(&mut deck, &setup(1, 1), &p, &mut 1);
    assert_eq!((days[0].mature, days[0].learning, days[0].new), (1, 1, 0));
    assert!((days[0].seconds - 75.0).abs() < 1e-3, "30 s + 45 s");
    assert_eq!(deck[0].last_day, 100);
    assert!(deck[2].memory.is_none(), "no new cards at 0 a day");
}

#[test]
fn new_cards_start_at_the_pace_given_and_stop_when_none_are_left() {
    let p = params_default();
    let mut deck = vec![new_card(); 3];
    let s = SimSetup {
        new_per_day: 0.5,
        ..setup(10, 1)
    };
    let days = rollout(&mut deck, &s, &p, &mut 3);
    let new: Vec<u32> = days.iter().map(|d| d.new).collect();
    assert_eq!(new, [0, 1, 0, 1, 0, 1, 0, 0, 0, 0]);
    assert!(deck.iter().all(|c| c.memory.is_some()));
}

#[test]
fn a_missed_day_moves_its_reviews_to_the_next_one() {
    let p = params_default();
    let s = SimSetup {
        miss_probability: 1.0,
        ..setup(5, 1)
    };
    let mut deck = [fsrs(10.0, 90, 100)];
    let days = rollout(&mut deck, &s, &p, &mut 1);
    assert!(days.iter().all(|d| day_total(d) == 0), "every day missed");
    assert_eq!(deck[0].last_day, 90);
}

#[test]
fn about_one_in_ten_due_cards_is_forgotten_and_relearned_the_next_day() {
    let p = params_default();
    // stability 10, last seen 10 days ago: recall 0.9 today
    let deck = vec![fsrs(10.0, 90, 100); 1_000];
    let f = simulate_forecast(&deck, &setup(3, 4), &p);
    assert!((load_total(&f[0]) - 1_000.0).abs() < 1e-3);
    assert!((f[0].minutes - 500.0).abs() < 1e-3, "1 000 reviews of 30 s");
    assert!((60.0..140.0).contains(&f[1].learning), "{:?}", f[1]);
}

#[test]
fn sm2_cards_are_carried_forward_too() {
    let p = params_default();
    let card = SimCard {
        memory: Some(SimMemory::Sm2(Sm2State {
            ease: 2.5,
            repetitions: 3,
            interval_days: 15,
        })),
        ..fsrs(1.0, 85, 100)
    };
    let mut deck = [card];
    let days = rollout(&mut deck, &setup(1, 1), &p, &mut 1);
    assert_eq!(days[0].young, 1, "an interval of 15 days is young");
    assert!(
        matches!(deck[0].memory, Some(SimMemory::Sm2(s)) if s.repetitions == 4 || s.repetitions == 0)
    );
}

#[test]
fn the_pacer_spreads_the_unseen_cards_over_the_days_before_the_buffer() {
    let p = params_default();
    let mut cards = vec![new_card(); 30];
    cards.push(fsrs(1.0, 95, 96)); // seen, will be forgotten by the exam without a review
    let i = pacer_input(100, 113, MIX);
    let pace = exam_pace(&cards, &i, &p).unwrap();
    assert_eq!((pace.days_left, pace.study_days), (13, 10));
    assert_eq!((pace.unseen_items, pace.volatile_items), (30, 1));
    assert_eq!(pace.required_new_per_day, 3);
    assert!(pace.on_track, "{pace:?}");
    assert!(pace.ready_share >= 0.9 && pace.reviews_per_day > 3.0 && pace.minutes_per_day > 0.0);
    assert!(exam_pace(&cards, &pacer_input(100, 100, MIX), &p).is_none());
}

#[test]
fn simulations_with_the_same_seed_agree() {
    let p = params_default();
    let deck: Vec<SimCard> = (0..50)
        .map(|i| fsrs(1.0 + i as f32, 100 - i, 100 + i % 5))
        .collect();
    let s = SimSetup {
        miss_probability: 0.2,
        new_per_day: 2.0,
        ..setup(30, 4)
    };
    assert_eq!(
        simulate_forecast(&deck, &s, &p),
        simulate_forecast(&deck, &s, &p)
    );
}

proptest! {
    #[test]
    fn the_band_is_ordered_and_never_negative(
        cards in proptest::collection::vec((0.5_f32..200.0, 0_i64..60, 0_i64..20), 0..40),
        miss in 0.0_f32..0.5,
    ) {
        let p = params_default();
        let deck: Vec<SimCard> = cards.iter().map(|(s, ago, due)| fsrs(*s, 100 - ago, 100 + due)).collect();
        let s = SimSetup { miss_probability: miss, ..setup(20, 5) };
        for d in simulate_forecast(&deck, &s, &p) {
            prop_assert!(d.p10 <= d.p50 && d.p50 <= d.p90);
            prop_assert!(d.new >= 0.0 && d.young >= 0.0 && d.minutes >= 0.0);
            prop_assert!(d.minutes_p10 <= d.minutes_p90);
        }
    }
}
