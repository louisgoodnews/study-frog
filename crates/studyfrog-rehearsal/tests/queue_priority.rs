#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Priority and the owner's difficulty in the queue.

use studyfrog_core::event::Difficulty;
use studyfrog_core::plan::{
    ItemKind, Order, plan_hardest, plan_sequential, plan_spaced, plan_stalest,
};
use studyfrog_core::priority::Priority;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{
    Budget, Budgets, Candidate, budget_unlimited, budgets_unlimited,
};
use studyfrog_srs::params::params_default;

const NOW: UnixMs = UnixMs(1_800_000_000_000);
const HOUR: i64 = 3_600_000;

fn item(uid: u128, priority: Priority, due_in_hours: Option<i64>) -> Candidate<'static> {
    Candidate {
        uid: Uid(uid),
        kind: ItemKind::Card,
        stack: Uid(1),
        tags: &[],
        position: uid as u32,
        memory: None,
        due: due_in_hours.map(|h| UnixMs(NOW.0 + h * HOUR)),
        last_review: due_in_hours.map(|_| UnixMs(NOW.0 - 100 * HOUR)),
        priority,
        difficulty_hint: None,
        familiarity: None,
    }
}

fn ids(q: &[Uid]) -> Vec<u128> {
    q.iter().map(|u| u.0).collect()
}

fn spaced(items: &[Candidate], budgets: &Budgets) -> Vec<Uid> {
    build_queue(items, &plan_spaced(), NOW, budgets, &params_default())
}

#[test]
fn spaced_runs_put_the_more_important_items_first_and_new_items_by_priority_too() {
    let items = [
        item(1, Priority::Low, Some(-48)),
        item(2, Priority::Highest, Some(-1)),
        item(3, Priority::Medium, Some(-24)),
        item(4, Priority::Low, None),
        item(5, Priority::High, None),
    ];
    assert_eq!(ids(&spaced(&items, &budgets_unlimited())), [2, 3, 1, 5, 4]);
}

#[test]
fn a_daily_allowance_goes_to_the_most_important_items() {
    let items = [
        item(1, Priority::Low, Some(-5)),
        item(2, Priority::Highest, Some(-1)),
        item(3, Priority::High, Some(-2)),
    ];
    let budgets = Budgets {
        global: Budget {
            new_items: None,
            reviews: Some(2),
        },
        per_stack: Default::default(),
    };
    assert_eq!(ids(&spaced(&items, &budgets)), [2, 3]);
}

#[test]
fn high_priorities_come_due_early_and_the_lowest_late() {
    let items = [
        item(1, Priority::Medium, Some(6)),
        item(2, Priority::High, Some(6)),
        item(3, Priority::Highest, Some(20)),
        item(4, Priority::Lowest, Some(-6)),
        item(5, Priority::Lowest, Some(-13)),
    ];
    assert_eq!(
        ids(&spaced(&items, &budgets_unlimited())),
        [3, 2, 5],
        "medium is not due, the lowest waits 12 hours"
    );
}

#[test]
fn priority_breaks_ties_in_the_other_orders_but_sequential_keeps_the_library_order() {
    let items = [
        item(1, Priority::Low, Some(-1)),
        item(2, Priority::Highest, Some(-1)),
        item(3, Priority::Medium, Some(-1)),
    ];
    let stale = build_queue(
        &items,
        &plan_stalest(),
        NOW,
        &budgets_unlimited(),
        &params_default(),
    );
    assert_eq!(
        ids(&stale),
        [2, 3, 1],
        "same last review: the more important first"
    );
    let seq = build_queue(
        &items,
        &plan_sequential(),
        NOW,
        &budgets_unlimited(),
        &params_default(),
    );
    assert_eq!(ids(&seq), [1, 2, 3]);
}

#[test]
fn the_owners_difficulty_stands_in_for_the_memory_of_unreviewed_items() {
    let mut hard = item(1, Priority::Medium, None);
    hard.difficulty_hint = Some(Difficulty::Difficult);
    let mut easy = item(2, Priority::Medium, None);
    easy.difficulty_hint = Some(Difficulty::Easy);
    let plain = item(3, Priority::Medium, None);
    let items = [easy, plain, hard];
    let q = build_queue(
        &items,
        &plan_hardest(),
        NOW,
        &budgets_unlimited(),
        &params_default(),
    );
    assert_eq!(
        ids(&q),
        [1, 2, 3],
        "difficult, then easy, then the unlabelled"
    );
    let mut easiest = plan_hardest();
    easiest.order = Order::EasiestFirst;
    let q = build_queue(
        &items,
        &easiest,
        NOW,
        &Budgets {
            global: budget_unlimited(),
            per_stack: Default::default(),
        },
        &params_default(),
    );
    assert_eq!(ids(&q), [2, 1, 3]);
}
