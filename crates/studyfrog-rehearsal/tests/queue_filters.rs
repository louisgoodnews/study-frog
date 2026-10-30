#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Queue filters, checked against an independent oracle.

mod common;

use common::queue_gen::{NOW, Owned, TAGS, borrow, candidates};
use proptest::prelude::*;
use studyfrog_core::plan::{
    DifficultyRange, ItemFilter, ItemKind, Order, RunPlan, StateFilter, filter_all, plan_sequential,
};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_srs::params::params_default;

fn run(items: &[Owned], filter: ItemFilter) -> Vec<u128> {
    let plan = RunPlan {
        filter,
        order: Order::Sequential,
        ..plan_sequential()
    };
    build_queue(
        &borrow(items),
        &plan,
        NOW,
        budget_unlimited(),
        &params_default(),
    )
    .iter()
    .map(|u: &Uid| u.0)
    .collect()
}

/// Straightforward restatement of the rules, written independently of the implementation.
fn oracle(o: &Owned, f: &ItemFilter) -> bool {
    let reviewed = o.last_review.is_some() || o.due.is_some() || o.memory.is_some();
    let due = reviewed && o.due.is_none_or(|d| d <= NOW);
    let state = match f.state {
        StateFilter::Any => true,
        StateFilter::New => !reviewed,
        StateFilter::Learned => reviewed,
        StateFilter::Due => due,
        StateFilter::NotDue => reviewed && !due,
    };
    let tags = f.tags_any.is_empty()
        || o.tags.iter().any(|t| {
            f.tags_any
                .iter()
                .any(|w| t.to_lowercase() == w.to_lowercase())
        });
    let difficulty = f.difficulty.is_none_or(|r| {
        o.memory
            .is_some_and(|m| m.difficulty >= r.min && m.difficulty <= r.max)
    });
    state
        && tags
        && difficulty
        && (f.kinds.is_empty() || f.kinds.contains(&o.kind))
        && (f.stacks.is_empty() || f.stacks.contains(&Uid(o.stack)))
}

fn filter() -> impl Strategy<Value = ItemFilter> {
    let kinds = prop::sample::subsequence(
        vec![ItemKind::Note, ItemKind::Card, ItemKind::Question],
        0..3,
    );
    let stacks = prop::collection::vec((1_u128..4).prop_map(Uid), 0..3);
    let tags = prop::sample::subsequence(vec!["ALPHA", "beta", "äpfel", "missing"], 0..3);
    let state = prop_oneof![
        Just(StateFilter::Any),
        Just(StateFilter::New),
        Just(StateFilter::Learned),
        Just(StateFilter::Due),
        Just(StateFilter::NotDue)
    ];
    let difficulty = prop::option::of(
        (1.0_f32..10.0, 0.0_f32..5.0).prop_map(|(min, w)| DifficultyRange { min, max: min + w }),
    );
    (kinds, stacks, tags, state, difficulty).prop_map(|(kinds, stacks, tags, state, difficulty)| {
        ItemFilter {
            kinds,
            stacks,
            tags_any: tags.into_iter().map(str::to_owned).collect(),
            state,
            difficulty,
        }
    })
}

proptest! {
    #[test]
    fn admitted_items_are_exactly_those_the_oracle_accepts(items in candidates(), f in filter()) {
        let expected: Vec<u128> = items.iter().filter(|o| oracle(o, &f)).map(|o| o.uid).collect();
        let mut got = run(&items, f);
        got.sort_unstable();
        let mut expected = expected;
        expected.sort_unstable();
        prop_assert_eq!(got, expected);
    }

    #[test]
    fn default_filter_admits_everything(items in candidates()) {
        prop_assert_eq!(run(&items, filter_all()).len(), items.len());
    }
}

#[test]
fn tags_match_case_insensitively_including_non_ascii() {
    let item = |uid, tag: &str| Owned {
        uid,
        kind: ItemKind::Note,
        stack: 1,
        tags: vec![tag.to_owned()],
        position: uid as u32,
        memory: None,
        due: None,
        last_review: None,
    };
    let items = [item(1, TAGS[2]), item(2, "gamma")];
    let f = ItemFilter {
        tags_any: vec!["äpfel".into()],
        ..filter_all()
    };
    assert_eq!(run(&items, f), [1]);
}

#[test]
fn difficulty_filter_excludes_items_without_fsrs_state() {
    let new = Owned {
        uid: 1,
        kind: ItemKind::Card,
        stack: 1,
        tags: vec![],
        position: 0,
        memory: None,
        due: None,
        last_review: None,
    };
    let f = ItemFilter {
        difficulty: Some(DifficultyRange {
            min: 1.0,
            max: 10.0,
        }),
        ..filter_all()
    };
    assert!(run(&[new], f).is_empty());
}
