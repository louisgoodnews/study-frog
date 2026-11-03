#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity in the queue: the least familiar first, a bound, and the daily allowance going to the least familiar.

use studyfrog_core::plan::{ItemFilter, ItemKind, Order, filter_all, plan_sequential, plan_spaced};
use studyfrog_core::priority::Priority;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budget, Budgets, Candidate, budgets_unlimited};
use studyfrog_srs::params::params_default;

const NOW: UnixMs = UnixMs(1_800_000_000_000);
const HOUR: i64 = 3_600_000;

fn item(uid: u128, fam: Option<f32>, due_in_hours: i64) -> Candidate<'static> {
    Candidate {
        uid: Uid(uid),
        kind: ItemKind::Card,
        stack: Uid(1),
        also_in: &[],
        tags: &[],
        position: uid as u32,
        memory: None,
        due: Some(UnixMs(NOW.0 + due_in_hours * HOUR)),
        last_review: Some(UnixMs(NOW.0 - 100 * HOUR)),
        priority: Priority::Medium,
        difficulty_hint: None,
        familiarity: fam,
    }
}

fn ids(q: &[Uid]) -> Vec<u128> {
    q.iter().map(|u| u.0).collect()
}

fn items() -> [Candidate<'static>; 4] {
    [
        item(1, Some(0.9), -50),
        item(2, Some(0.2), -1),
        item(3, None, -10),
        item(4, Some(0.5), -30),
    ]
}

#[test]
fn the_least_familiar_order_puts_unknown_items_first() {
    let mut plan = plan_sequential();
    plan.order = Order::LeastFamiliar;
    let q = build_queue(
        &items(),
        &plan,
        NOW,
        &budgets_unlimited(),
        &params_default(),
    );
    assert_eq!(ids(&q), [3, 2, 4, 1]);
}

#[test]
fn the_bound_keeps_the_less_familiar_and_unknown_items() {
    let mut plan = plan_sequential();
    plan.filter = ItemFilter {
        familiarity_below: Some(0.5),
        ..filter_all()
    };
    let q = build_queue(
        &items(),
        &plan,
        NOW,
        &budgets_unlimited(),
        &params_default(),
    );
    assert_eq!(ids(&q), [2, 3], "0.5 itself is not below 0.5");
}

#[test]
fn a_daily_allowance_goes_to_the_least_familiar_due_items() {
    let budgets = Budgets {
        global: Budget {
            new_items: None,
            reviews: Some(2),
        },
        per_stack: Default::default(),
    };
    let q = build_queue(&items(), &plan_spaced(), NOW, &budgets, &params_default());
    assert_eq!(ids(&q), [3, 2], "not the most overdue ones");
}
