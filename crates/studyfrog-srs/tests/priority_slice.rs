#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Continuous priority queue and anti-paralysis scoring tests.

use proptest::prelude::*;
use studyfrog_srs::priority_slice::{
    PriorityWeights, calculate_effective_priority, default_priority_weights, rank_by_priority,
    slice_by_budget,
};

#[test]
fn default_weights_have_expected_distribution() {
    let weights = default_priority_weights();
    assert!((weights.priority_weight - 0.40).abs() < 1e-6);
    assert!((weights.retrievability_weight - 0.35).abs() < 1e-6);
    assert!((weights.familiarity_weight - 0.25).abs() < 1e-6);
    let sum = weights.priority_weight + weights.retrievability_weight + weights.familiarity_weight;
    assert!((sum - 1.0).abs() < 1e-6);

    let default_trait: PriorityWeights = Default::default();
    assert_eq!(weights, default_trait);
}

#[test]
fn bounds_extremes() {
    let weights = default_priority_weights();

    // Max priority, min retrievability (completely forgotten), min familiarity (brand new)
    let max_score = calculate_effective_priority(1.0, 0.0, 0.0, &weights);
    assert!((max_score - 1.0).abs() < 1e-6);

    // Min priority, max retrievability (fully retained), max familiarity (mastered)
    let min_score = calculate_effective_priority(0.0, 1.0, 1.0, &weights);
    assert!((min_score - 0.0).abs() < 1e-6);
}

#[test]
fn exact_formula_verification() {
    let weights = default_priority_weights();
    // Formula: 0.40 * 0.5 + 0.35 * (1.0 - 0.8) + 0.25 * (1.0 - 0.6)
    //        = 0.20 + 0.35 * 0.2 + 0.25 * 0.4
    //        = 0.20 + 0.07 + 0.10 = 0.37
    let score = calculate_effective_priority(0.5, 0.8, 0.6, &weights);
    assert!((score - 0.37).abs() < 1e-6);
}

#[test]
fn input_and_output_clamping() {
    let weights = default_priority_weights();

    // Inputs outside [0.0, 1.0] are clamped
    let clamped_high = calculate_effective_priority(2.5, -0.5, -1.0, &weights);
    assert!((clamped_high - 1.0).abs() < 1e-6);

    let clamped_low = calculate_effective_priority(-1.0, 1.5, 2.0, &weights);
    assert!((clamped_low - 0.0).abs() < 1e-6);

    // Extreme weights clamping
    let high_weights = PriorityWeights {
        priority_weight: 2.0,
        retrievability_weight: 2.0,
        familiarity_weight: 2.0,
    };
    let score_high = calculate_effective_priority(1.0, 0.0, 0.0, &high_weights);
    assert_eq!(score_high, 1.0);

    let neg_weights = PriorityWeights {
        priority_weight: -1.0,
        retrievability_weight: -1.0,
        familiarity_weight: -1.0,
    };
    let score_low = calculate_effective_priority(1.0, 0.0, 0.0, &neg_weights);
    assert_eq!(score_low, 0.0);
}

#[test]
fn nan_inputs_handled_safely() {
    let weights = default_priority_weights();
    let score = calculate_effective_priority(f64::NAN, f64::NAN, f64::NAN, &weights);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn monotonicity_of_components() {
    let weights = default_priority_weights();

    // Higher user priority -> higher score
    let s1 = calculate_effective_priority(0.2, 0.5, 0.5, &weights);
    let s2 = calculate_effective_priority(0.8, 0.5, 0.5, &weights);
    assert!(s2 > s1);

    // Lower retrievability (higher forgetting risk) -> higher urgency score
    let r_fresh = calculate_effective_priority(0.5, 0.9, 0.5, &weights);
    let r_decayed = calculate_effective_priority(0.5, 0.2, 0.5, &weights);
    assert!(r_decayed > r_fresh);

    // Lower familiarity (newer card) -> higher urgency score
    let f_mature = calculate_effective_priority(0.5, 0.5, 0.9, &weights);
    let f_novel = calculate_effective_priority(0.5, 0.5, 0.1, &weights);
    assert!(f_novel > f_mature);
}

#[derive(Debug, Clone, PartialEq)]
struct MockCard {
    id: u32,
    priority: f64,
    retrievability: f64,
    familiarity: f64,
}

#[test]
fn ranking_and_slicing() {
    let weights = default_priority_weights();
    let mut cards = vec![
        MockCard {
            id: 1, // low priority, high retention, mature -> lowest urgency
            priority: 0.1,
            retrievability: 0.95,
            familiarity: 0.9,
        },
        MockCard {
            id: 2, // high priority, low retention -> highest urgency
            priority: 0.9,
            retrievability: 0.1,
            familiarity: 0.1,
        },
        MockCard {
            id: 3, // medium
            priority: 0.5,
            retrievability: 0.5,
            familiarity: 0.5,
        },
    ];

    rank_by_priority(&mut cards, |c| {
        calculate_effective_priority(c.priority, c.retrievability, c.familiarity, &weights)
    });

    assert_eq!(cards[0].id, 2);
    assert_eq!(cards[1].id, 3);
    assert_eq!(cards[2].id, 1);

    // Budget slicing
    let sliced = slice_by_budget(&cards, 2, |c| {
        calculate_effective_priority(c.priority, c.retrievability, c.familiarity, &weights)
    });
    assert_eq!(sliced.len(), 2);
    assert_eq!(sliced[0].id, 2);
    assert_eq!(sliced[1].id, 3);
}

#[test]
fn ranking_preserves_stable_order_for_ties() {
    let mut cards = vec![
        MockCard {
            id: 10,
            priority: 0.5,
            retrievability: 0.5,
            familiarity: 0.5,
        },
        MockCard {
            id: 20,
            priority: 0.5,
            retrievability: 0.5,
            familiarity: 0.5,
        },
        MockCard {
            id: 30,
            priority: 0.5,
            retrievability: 0.5,
            familiarity: 0.5,
        },
    ];

    rank_by_priority(&mut cards, |_| 0.5);
    assert_eq!(cards[0].id, 10);
    assert_eq!(cards[1].id, 20);
    assert_eq!(cards[2].id, 30);
}

proptest! {
    #[test]
    fn effective_priority_always_in_unit_interval(
        p in -2.0..2.0f64,
        r in -2.0..2.0f64,
        f in -2.0..2.0f64,
        wp in 0.0..1.0f64,
        wr in 0.0..1.0f64,
        wf in 0.0..1.0f64,
    ) {
        let weights = PriorityWeights {
            priority_weight: wp,
            retrievability_weight: wr,
            familiarity_weight: wf,
        };
        let score = calculate_effective_priority(p, r, f, &weights);
        prop_assert!((0.0..=1.0).contains(&score));
    }
}
