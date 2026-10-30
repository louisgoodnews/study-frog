#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Generators for queue-builder tests.

use proptest::prelude::*;
use studyfrog_core::memory::MemState;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::candidate::Candidate;

pub const NOW: UnixMs = UnixMs(1_800_000_000_000);
const DAY: i64 = 86_400_000;
pub const TAGS: [&str; 4] = ["alpha", "Beta", "ÄPFEL", "gamma"];

/// A candidate that owns its tags.
#[derive(Debug, Clone)]
pub struct Owned {
    pub uid: u128,
    pub kind: ItemKind,
    pub stack: u128,
    pub tags: Vec<String>,
    pub position: u32,
    pub memory: Option<MemState>,
    pub due: Option<UnixMs>,
    pub last_review: Option<UnixMs>,
}

pub fn borrow(items: &[Owned]) -> Vec<Candidate<'_>> {
    items
        .iter()
        .map(|o| Candidate {
            uid: Uid(o.uid),
            kind: o.kind,
            stack: Uid(o.stack),
            tags: &o.tags,
            position: o.position,
            memory: o.memory,
            due: o.due,
            last_review: o.last_review,
        })
        .collect()
}

fn kind() -> impl Strategy<Value = ItemKind> {
    prop_oneof![
        Just(ItemKind::Note),
        Just(ItemKind::Card),
        Just(ItemKind::Question)
    ]
}

/// (new | reviewed-with-FSRS | reviewed-without-FSRS) schedule data.
fn schedule() -> impl Strategy<Value = (Option<MemState>, Option<UnixMs>, Option<UnixMs>)> {
    let state = (0.5_f32..100.0, 1.0_f32..10.0).prop_map(|(stability, difficulty)| MemState {
        stability,
        difficulty,
    });
    let when = (-30_i64..30, 1_i64..90)
        .prop_map(|(due, ago)| (UnixMs(NOW.0 + due * DAY), UnixMs(NOW.0 - ago * DAY)));
    prop_oneof![
        Just((None, None, None)),
        (state, when.clone()).prop_map(|(m, (d, l))| (Some(m), Some(d), Some(l))),
        when.prop_map(|(d, l)| (None, Some(d), Some(l))),
    ]
}

/// Up to 40 candidates with unique uids and positions, listed in a random order.
pub fn candidates() -> impl Strategy<Value = Vec<Owned>> {
    let one = (
        kind(),
        1_u128..4,
        prop::sample::subsequence(TAGS.to_vec(), 0..3),
        schedule(),
    );
    prop::collection::vec(one, 0..40).prop_flat_map(|items| {
        let n = items.len();
        let order = Just((0..n).collect::<Vec<_>>()).prop_shuffle();
        (Just(items), order).prop_map(|(items, order)| {
            let mut owned: Vec<Owned> = items
                .into_iter()
                .enumerate()
                .map(
                    |(i, (kind, stack, tags, (memory, due, last_review)))| Owned {
                        uid: i as u128 + 1,
                        kind,
                        stack,
                        tags: tags.into_iter().map(str::to_owned).collect(),
                        position: i as u32,
                        memory,
                        due,
                        last_review,
                    },
                )
                .collect();
            let by_position = order.clone();
            owned.sort_by_key(|o| by_position[o.position as usize]);
            owned
        })
    })
}

/// The candidate with `uid`; panics if absent (tests only).
pub fn find(items: &[Owned], uid: Uid) -> &Owned {
    items.iter().find(|o| Uid(o.uid) == uid).unwrap()
}
