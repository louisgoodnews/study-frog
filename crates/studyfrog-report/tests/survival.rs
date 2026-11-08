#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! True retention and forgetting curves by stability tier (STORY-0107).

use proptest::prelude::*;
use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_report::retention_survival::{
    SurvivalReview, half_life_days, predicted_of, rate_of, survival_report,
};
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state, retrievability};
use studyfrog_srs::params::params_default;

fn rv(item: u64, rating: Rating, elapsed_days: f32) -> SurvivalReview {
    SurvivalReview {
        item,
        rating,
        elapsed_days,
    }
}

#[test]
fn an_empty_history_has_four_tiers_of_five_bins_and_no_numbers() {
    let r = survival_report(&[], &params_default());
    assert_eq!(r.tiers.len(), 4);
    assert!(
        r.tiers
            .iter()
            .all(|t| t.bins.len() == 5 && t.rmse.is_none())
    );
    assert_eq!(rate_of(&r.true_retention), None);
    assert_eq!((r.memory_scale, r.half_life_days), (None, None));
}

#[test]
fn a_young_review_is_counted_with_the_fsrs_prediction() {
    let p = params_default();
    let r = survival_report(&[rv(1, Rating::Good, 0.0), rv(1, Rating::Again, 5.0)], &p);
    let first = initial_state(Rating::Good, &p);
    assert!(first.stability < 7.0, "a new item is in the first tier");
    assert_eq!((r.young.reviews, r.young.recalled), (1, 0));
    assert_eq!(r.mature.reviews, 0);
    let want = retrievability(&first, 5.0, &p);
    assert!((predicted_of(&r.young).unwrap() - want).abs() < 1e-6);
    assert_eq!(r.tiers[0].total.reviews, 1);
    // measured 0 %, predicted `want`: the error of the only bin is `want`
    assert!((r.tiers[0].rmse.unwrap() - want).abs() < 1e-6);
}

#[test]
fn same_day_reviews_change_the_state_but_are_not_counted() {
    let r = survival_report(
        &[rv(1, Rating::Good, 0.0), rv(1, Rating::Good, 0.2)],
        &params_default(),
    );
    assert_eq!(r.young.reviews + r.mature.reviews, 0);
}

#[test]
fn reviews_on_their_due_day_of_a_mature_item_are_true_retention() {
    let p = params_default();
    let (mut s, mut reviews, mut mature_on_time) = (
        initial_state(Rating::Good, &p),
        vec![rv(7, Rating::Good, 0.0)],
        0,
    );
    for _ in 0..8 {
        let gap = interval_days(&s, &p) as f32;
        mature_on_time += u32::from(s.stability >= 21.0);
        reviews.push(rv(7, Rating::Good, gap));
        s = next_state(&s, Rating::Good, gap, &p);
    }
    let r = survival_report(&reviews, &p);
    assert!(mature_on_time >= 2, "the item matures within eight reviews");
    assert_eq!(r.true_retention.reviews, mature_on_time);
    assert_eq!(rate_of(&r.true_retention), Some(1.0));
    assert_eq!(r.on_time.reviews, 8, "every review was on its due day");
    assert_eq!(r.early.reviews + r.overdue.reviews, 0);
}

#[test]
fn an_overdue_review_is_counted_as_overdue() {
    let p = params_default();
    let gap = interval_days(&initial_state(Rating::Good, &p), &p) as f32 * 3.0;
    let r = survival_report(&[rv(1, Rating::Good, 0.0), rv(1, Rating::Good, gap)], &p);
    assert_eq!((r.overdue.reviews, r.on_time.reviews), (1, 0));
}

#[test]
fn the_half_life_is_where_the_model_predicts_one_half() {
    let p = params_default();
    for s in [1.0_f32, 10.0, 100.0] {
        let h = half_life_days(s, &p);
        let at = retrievability(
            &MemState {
                stability: s,
                difficulty: 5.0,
            },
            h,
            &p,
        );
        assert!((at - 0.5).abs() < 1e-3, "{s}: {at}");
    }
}

#[test]
fn a_learner_who_follows_fsrs_has_a_memory_scale_near_one() {
    let p = params_default();
    let history = simulate_history(
        &SimConfig {
            seed: 5,
            days: 200,
            new_per_day: 10,
            miss_percent: 20,
        },
        &p,
    );
    let mut last = std::collections::HashMap::new();
    let reviews: Vec<SurvivalReview> = history
        .iter()
        .map(|r| {
            let elapsed = last
                .insert(r.item, r.day)
                .map_or(0.0, |d| (r.day - d) as f32);
            rv(r.item, r.rating, elapsed)
        })
        .collect();
    let r = survival_report(&reviews, &p);
    let scale = r.memory_scale.unwrap();
    assert!((0.7..1.45).contains(&scale), "{scale}");
    assert!(r.half_life_days.unwrap() > 0.0);
    let t = rate_of(&r.true_retention).unwrap();
    assert!(
        (0.8..0.97).contains(&t),
        "true retention near the target: {t}"
    );
}

proptest! {
    #[test]
    fn every_spaced_review_is_counted_once_in_each_breakdown(
        steps in proptest::collection::vec((0_u64..5, 1_u8..=4, 0.0_f32..60.0), 0..80)
    ) {
        let mut seen = std::collections::HashSet::new();
        let mut spaced = 0;
        let reviews: Vec<SurvivalReview> = steps.iter().map(|(item, rating, gap)| {
            let first = seen.insert(*item);
            let gap = if first { 0.0 } else { *gap };
            spaced += u32::from(!first && gap >= 1.0);
            rv(*item, studyfrog_core::review::rating_from_value(*rating).unwrap(), gap)
        }).collect();
        let r = survival_report(&reviews, &params_default());
        prop_assert_eq!(r.young.reviews + r.mature.reviews, spaced);
        prop_assert_eq!(r.early.reviews + r.on_time.reviews + r.overdue.reviews, spaced);
        prop_assert_eq!(r.tiers.iter().map(|t| t.total.reviews).sum::<u32>(), spaced);
        let binned: u32 = r.tiers.iter().flat_map(|t| t.bins.iter()).map(|b| b.count.reviews).sum();
        prop_assert_eq!(binned, spaced);
        prop_assert!(r.true_retention.reviews <= r.mature.reviews);
    }
}
