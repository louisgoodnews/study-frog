#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Queue orders: permutation, determinism, and the ordering each one promises.

mod common;

use std::collections::HashSet;

use common::queue_gen::{NOW, Owned, borrow, candidates, find};
use proptest::prelude::*;
use std::collections::HashMap;
use studyfrog_core::plan::{Order, RunPlan, plan_shuffled, plan_spaced};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;

use studyfrog_rehearsal::queue::candidate::{Budget, Budgets, budget_unlimited};
use studyfrog_srs::fsrs::retrievability;
use studyfrog_srs::params::params_default;

const PRACTICE: [Order; 6] = [
    Order::Sequential,
    Order::Shuffled,
    Order::HardestFirst,
    Order::EasiestFirst,
    Order::StalestFirst,
    Order::LowestRetrievability,
];

fn plan(order: Order, seed: u64) -> RunPlan {
    RunPlan {
        order,
        seed,
        ..plan_shuffled(seed)
    }
}

fn queue(items: &[Owned], plan: &RunPlan, budget: Budget) -> Vec<Uid> {
    build_queue(
        &borrow(items),
        plan,
        NOW,
        &Budgets {
            global: budget,
            per_stack: HashMap::new(),
        },
        &params_default(),
    )
}

proptest! {
    #[test]
    fn practice_orders_visit_every_item_exactly_once(items in candidates(), seed in any::<u64>()) {
        let all: HashSet<u128> = items.iter().map(|o| o.uid).collect();
        for order in PRACTICE {
            let got = queue(&items, &plan(order, seed), budget_unlimited());
            prop_assert_eq!(got.len(), items.len(), "{:?}", order);
            prop_assert_eq!(got.iter().map(|u| u.0).collect::<HashSet<_>>(), all.clone());
        }
    }

    #[test]
    fn result_does_not_depend_on_input_order(items in candidates(), seed in any::<u64>()) {
        let mut reversed = items.clone();
        reversed.reverse();
        for order in PRACTICE.into_iter().chain([Order::Due]) {
            let p = plan(order, seed);
            prop_assert_eq!(queue(&items, &p, budget_unlimited()), queue(&reversed, &p, budget_unlimited()), "{:?}", order);
        }
    }

    #[test]
    fn sequential_follows_library_position(items in candidates()) {
        let got = queue(&items, &plan(Order::Sequential, 0), budget_unlimited());
        let positions: Vec<u32> = got.iter().map(|u| find(&items, *u).position).collect();
        prop_assert!(positions.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn difficulty_orders_put_unscheduled_items_last(items in candidates()) {
        for (order, sign) in [(Order::HardestFirst, -1.0_f32), (Order::EasiestFirst, 1.0)] {
            let got = queue(&items, &plan(order, 0), budget_unlimited());
            let keys: Vec<Option<f32>> = got.iter().map(|u| find(&items, *u).memory.map(|m| sign * m.difficulty)).collect();
            let scheduled = keys.iter().take_while(|k| k.is_some()).count();
            prop_assert!(keys[scheduled..].iter().all(Option::is_none));
            prop_assert!(keys[..scheduled].windows(2).all(|w| w[0] <= w[1]));
        }
    }

    #[test]
    fn stalest_first_starts_with_never_reviewed(items in candidates()) {
        let got = queue(&items, &plan(Order::StalestFirst, 0), budget_unlimited());
        let last: Vec<_> = got.iter().map(|u| find(&items, *u).last_review).collect();
        prop_assert!(last.windows(2).all(|w| w[0] <= w[1]), "None sorts before Some");
    }

    #[test]
    fn lowest_retrievability_ascends(items in candidates()) {
        let p = params_default();
        let got = queue(&items, &plan(Order::LowestRetrievability, 0), budget_unlimited());
        let recalls: Vec<f32> = got
            .iter()
            .filter_map(|u| {
                let o = find(&items, *u);
                let elapsed = o.last_review.map_or(0.0, |t| (NOW.0 - t.0) as f32 / 86_400_000.0);
                o.memory.map(|m| retrievability(&m, elapsed, &p))
            })
            .collect();
        prop_assert!(recalls.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn limit_caps_the_queue(items in candidates(), limit in 0_u32..20) {
        let p = RunPlan { limit: Some(limit), ..plan(Order::Sequential, 0) };
        prop_assert_eq!(queue(&items, &p, budget_unlimited()).len(), items.len().min(limit as usize));
    }

    #[test]
    fn spaced_puts_due_reviews_first_then_new_within_budget(
        items in candidates(),
        reviews in prop::option::of(0_u32..10),
        new_items in prop::option::of(0_u32..10),
    ) {
        let got = queue(&items, &plan_spaced(), Budget { new_items, reviews });
        let rows: Vec<&Owned> = got.iter().map(|u| find(&items, *u)).collect();
        let due: Vec<&&Owned> = rows.iter().filter(|o| o.due.is_some()).collect();
        let fresh = rows.len() - due.len();
        prop_assert!(rows.iter().take(due.len()).all(|o| o.due.is_some()), "reviews come first");
        prop_assert!(due.iter().all(|o| o.due.unwrap() <= NOW), "nothing that is not due");
        prop_assert!(due.windows(2).all(|w| w[0].due <= w[1].due), "most overdue first");
        prop_assert!(reviews.is_none_or(|r| due.len() <= r as usize));
        prop_assert!(new_items.is_none_or(|n| fresh <= n as usize));
        let available_due = items.iter().filter(|o| o.due.is_some_and(|d| d <= NOW)).count();
        let available_new = items.iter().filter(|o| o.due.is_none()).count();
        prop_assert_eq!(due.len(), available_due.min(reviews.map_or(usize::MAX, |r| r as usize)));
        prop_assert_eq!(fresh, available_new.min(new_items.map_or(usize::MAX, |n| n as usize)));
    }
}

#[test]
fn shuffle_is_reproducible_and_seed_dependent() {
    let items: Vec<Owned> = (0..20)
        .map(|i| Owned {
            uid: i + 1,
            kind: studyfrog_core::plan::ItemKind::Card,
            stack: 1,
            tags: Vec::new(),
            position: i as u32,
            memory: None,
            due: None,
            last_review: None,
        })
        .collect();
    let a = queue(&items, &plan(Order::Shuffled, 1), budget_unlimited());
    assert_eq!(
        a,
        queue(&items, &plan(Order::Shuffled, 1), budget_unlimited())
    );
    assert_ne!(
        a,
        queue(&items, &plan(Order::Shuffled, 2), budget_unlimited())
    );
    assert_ne!(
        a,
        queue(&items, &plan(Order::Sequential, 1), budget_unlimited())
    );
}
