#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Shuffling as a switch that goes with any order.

mod common;

use std::collections::{HashMap, HashSet};

use common::queue_gen::{NOW, Owned, borrow};
use studyfrog_core::plan::{Order, RunPlan, plan_shuffled};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budget, Budgets, budget_unlimited};
use studyfrog_srs::params::params_default;

fn plan(order: Order, seed: u64) -> RunPlan {
    RunPlan {
        order,
        seed,
        shuffle: false,
        ..plan_shuffled(seed)
    }
}

fn shuffled(order: Order, seed: u64) -> RunPlan {
    RunPlan {
        shuffle: true,
        ..plan(order, seed)
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

fn plain_items(n: u128) -> Vec<Owned> {
    (0..n)
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
        .collect()
}

#[test]
fn shuffling_any_order_keeps_the_chosen_items_and_only_mixes_their_sequence() {
    let items = plain_items(30);
    for order in [
        Order::Due,
        Order::HardestFirst,
        Order::StalestFirst,
        Order::Sequential,
    ] {
        let plain = queue(&items, &plan(order, 5), budget_unlimited());
        let mixed = queue(&items, &shuffled(order, 5), budget_unlimited());
        assert_eq!(
            mixed.iter().collect::<HashSet<_>>(),
            plain.iter().collect::<HashSet<_>>(),
            "{order:?}"
        );
        assert_eq!(
            mixed,
            queue(&items, &shuffled(order, 5), budget_unlimited()),
            "repeatable"
        );
        assert_ne!(mixed, plain, "{order:?} came out in the same order");
    }
}

#[test]
fn a_limit_cuts_by_priority_first_except_for_sequential_runs_where_it_samples() {
    let items = plain_items(30);
    let top = |order| {
        queue(
            &items,
            &RunPlan {
                limit: Some(5),
                ..plan(order, 9)
            },
            budget_unlimited(),
        )
    };
    let top_shuffled = |order| {
        queue(
            &items,
            &RunPlan {
                limit: Some(5),
                ..shuffled(order, 9)
            },
            budget_unlimited(),
        )
    };

    let spaced: HashSet<_> = top(Order::Due).into_iter().collect();
    assert_eq!(
        top_shuffled(Order::Due).into_iter().collect::<HashSet<_>>(),
        spaced,
        "the same five, mixed"
    );

    let first_five: HashSet<_> = top(Order::Sequential).into_iter().collect();
    let sample = top_shuffled(Order::Sequential);
    assert_eq!(sample.len(), 5);
    assert_ne!(
        sample.into_iter().collect::<HashSet<_>>(),
        first_five,
        "a random sample, not the first five"
    );
}
