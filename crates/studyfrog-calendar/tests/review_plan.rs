#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The planner: grouping, skipping and the time it takes.

use std::collections::HashMap;
use std::time::Instant;

use proptest::prelude::*;
use studyfrog_calendar::review_plan::{PlanItem, plan_entries};
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;

fn item(n: u128, stack: Option<u128>, due_day: i64, status: Status) -> PlanItem {
    PlanItem {
        uid: Uid(n),
        stack: stack.map(Uid),
        due_day,
        status,
        stack_skips: false,
    }
}

#[test]
fn items_group_by_day_and_stack_with_the_stack_size() {
    let items = [
        item(1, Some(100), 18, Status::Learning),
        item(2, Some(100), 18, Status::Reviewing),
        item(3, Some(100), 19, Status::Learning),
        item(4, Some(200), 18, Status::New),
        item(5, None, 18, Status::Learning),
    ];
    let sizes = HashMap::from([(Some(Uid(100)), 10), (Some(Uid(200)), 3)]);
    let got = plan_entries(&items, &sizes, (10, 30));
    let shape: Vec<_> = got
        .iter()
        .map(|e| (e.day, e.stack.map(|s| s.0), e.items.len(), e.stack_size))
        .collect();
    assert_eq!(
        shape,
        [
            (18, None, 1, 1),
            (18, Some(100), 2, 10),
            (18, Some(200), 1, 3),
            (19, Some(100), 1, 10)
        ]
    );
}

#[test]
fn overdue_items_land_on_the_first_day_and_the_window_ends_at_the_last() {
    let items = [
        item(1, Some(1), 3, Status::Learning),
        item(2, Some(1), 12, Status::Learning),
        item(3, Some(1), 31, Status::Learning),
    ];
    let got = plan_entries(&items, &HashMap::new(), (10, 30));
    assert_eq!(got.iter().map(|e| e.day).collect::<Vec<_>>(), [10, 12]);
    assert_eq!(got[0].items, [Uid(1)]);
}

#[test]
fn done_and_paused_items_and_skipped_stacks_make_no_entry() {
    let mut skipped = item(4, Some(1), 12, Status::Learning);
    skipped.stack_skips = true;
    let items = [
        item(1, Some(1), 12, Status::Done),
        item(2, Some(1), 12, Status::Paused),
        item(3, Some(1), 12, Status::Learning),
        skipped,
    ];
    let got = plan_entries(&items, &HashMap::new(), (10, 30));
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].items, [Uid(3)]);
}

#[test]
fn ten_thousand_items_plan_within_the_budget() {
    let items: Vec<PlanItem> = (0..10_000)
        .map(|i| item(i, Some(i % 40), 20_000 + (i % 90) as i64, Status::Learning))
        .collect();
    let sizes = HashMap::new();
    let start = Instant::now();
    let got = plan_entries(&items, &sizes, (20_000, 20_060));
    let took = start.elapsed();
    assert!(!got.is_empty());
    assert!(took.as_millis() < 100, "planning 10k items took {took:?}");
}

fn status_of(n: u8) -> Status {
    [
        Status::New,
        Status::Learning,
        Status::Reviewing,
        Status::Done,
        Status::Paused,
    ][usize::from(n % 5)]
}

proptest! {
    #[test]
    fn every_live_item_appears_once_and_no_skipped_one_does(
        rows in proptest::collection::vec((0u8..6, 0i64..60, 0u8..5, any::<bool>()), 0..200)
    ) {
        let items: Vec<PlanItem> = rows.iter().enumerate().map(|(i, (s, d, st, sk))| PlanItem {
            uid: Uid(i as u128),
            stack: (*s < 5).then_some(Uid(u128::from(*s))),
            due_day: *d,
            status: status_of(*st),
            stack_skips: *sk,
        }).collect();
        let got = plan_entries(&items, &HashMap::new(), (10, 50));
        let planned: Vec<Uid> = got.iter().flat_map(|e| e.items.clone()).collect();
        let mut unique = planned.clone();
        unique.sort();
        unique.dedup();
        prop_assert_eq!(planned.len(), unique.len());
        for it in &items {
            let live = !it.stack_skips && !matches!(it.status, Status::Done | Status::Paused) && it.due_day.max(10) <= 50;
            prop_assert_eq!(planned.contains(&it.uid), live);
        }
        prop_assert!(got.windows(2).all(|w| (w[0].day, w[0].stack) < (w[1].day, w[1].stack)));
    }
}
