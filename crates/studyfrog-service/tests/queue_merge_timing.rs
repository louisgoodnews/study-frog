#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0218: what merging an item's stacks costs when building a queue. Run by hand:
//! `cargo test --release -p studyfrog-service --test queue_merge_timing -- --ignored --nocapture`.

use std::collections::HashMap;
use std::time::Instant;

use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::Priority;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::queue::candidates;
use studyfrog_store::items::IndexedItem;
use studyfrog_store::schedule::Scheduled;

#[test]
#[ignore = "a measurement, run by hand in release mode"]
fn merging_ten_thousand_items_held_by_two_stacks() {
    let items: Vec<IndexedItem> = (0..10_000u128)
        .map(|n| IndexedItem {
            uid: Uid(n + 1),
            kind: ItemKind::Card,
            stack: Uid(1 + n % 20),
            also_in: vec![Uid(21 + n % 20)],
            tags: Vec::new(),
            position: n as u32,
            priority: Priority::Medium,
            difficulty: None,
        })
        .collect();
    let mut sched = HashMap::new();
    for it in &items {
        for s in std::iter::once(it.stack).chain(it.also_in.iter().copied()) {
            let due = UnixMs(1_000 + (it.uid.0 as i64 * 7 + s.0 as i64) % 5_000);
            sched.insert(
                (it.uid, s),
                Scheduled {
                    memory: None,
                    due,
                    last_review: UnixMs(0),
                },
            );
        }
    }
    let none: &[Uid] = &[];
    let start = Instant::now();
    let runs = 20;
    for _ in 0..runs {
        let c = candidates(&items, (&sched, none), &HashMap::new());
        assert_eq!(c.len(), 10_000);
    }
    println!(
        "candidates for 10k items in 2 stacks each: {:?} per build",
        start.elapsed() / runs
    );
}
