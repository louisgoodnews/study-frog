#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Per-stack and global budgets of the spaced order.

mod common;

use std::collections::HashMap;

use common::queue_gen::{NOW, Owned, borrow, find};
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budget, Budgets, budget_unlimited};
use studyfrog_srs::params::params_default;

fn spaced_items() -> Vec<Owned> {
    // Two stacks that alternate by uid; the first six items are due (most overdue = highest uid), the rest new.
    let mk = |uid: u128, due: Option<i64>| Owned {
        uid,
        kind: studyfrog_core::plan::ItemKind::Card,
        stack: 1 + (uid % 2),
        tags: Vec::new(),
        position: uid as u32,
        memory: None,
        due: due.map(|d| studyfrog_core::time::UnixMs(NOW.0 + d)),
        last_review: due.map(|_| studyfrog_core::time::UnixMs(NOW.0 - 1)),
    };
    (1..=14)
        .map(|i| mk(i, (i <= 6).then_some(-(i as i64))))
        .collect()
}

#[test]
fn per_stack_budgets_cap_each_stack_then_the_global_cap_applies() {
    let items = spaced_items();
    let budgets = Budgets {
        global: Budget {
            new_items: Some(3),
            reviews: None,
        },
        per_stack: HashMap::from([
            (
                Uid(1),
                Budget {
                    new_items: Some(2),
                    reviews: Some(1),
                },
            ),
            (
                Uid(2),
                Budget {
                    new_items: None,
                    reviews: Some(2),
                },
            ),
        ]),
    };
    let got = build_queue(
        &borrow(&items),
        &plan_spaced(),
        NOW,
        &budgets,
        &params_default(),
    );
    let (due, new): (Vec<Uid>, Vec<Uid>) =
        got.iter().partition(|u| find(&items, **u).due.is_some());
    let in_stack = |list: &[Uid], stack: u128| {
        list.iter()
            .filter(|u| find(&items, **u).stack == stack)
            .count()
    };
    assert_eq!(
        (in_stack(&due, 1), in_stack(&due, 2)),
        (1, 2),
        "review caps per stack"
    );
    assert!(in_stack(&new, 1) <= 2, "new cap of stack 1");
    assert_eq!(
        new.len(),
        3,
        "the global new cap applies after the stack caps"
    );
    assert!(
        due.windows(2)
            .all(|w| find(&items, w[0]).due <= find(&items, w[1]).due),
        "still most overdue first"
    );
}

#[test]
fn a_stack_with_no_allowance_left_contributes_nothing() {
    let items = spaced_items();
    let none = Budget {
        new_items: Some(0),
        reviews: Some(0),
    };
    let budgets = Budgets {
        global: budget_unlimited(),
        per_stack: HashMap::from([(Uid(1), none)]),
    };
    let got = build_queue(
        &borrow(&items),
        &plan_spaced(),
        NOW,
        &budgets,
        &params_default(),
    );
    assert!(!got.is_empty() && got.iter().all(|u| find(&items, *u).stack == 2));
}
