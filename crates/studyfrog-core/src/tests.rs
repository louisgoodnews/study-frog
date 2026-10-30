//! Property tests for ids and the arena.

use std::collections::HashSet;

use proptest::prelude::*;

use crate::arena::{Arena, arena_get, arena_get_mut, arena_iter, arena_len, arena_new, arena_push};
use crate::idx::{Idx, idx};

/// Deliberately implements no traits: indices must not require any bound on `T`.
struct Opaque;

fn filled(values: &[i64]) -> (Arena<i64>, Vec<Idx<i64>>) {
    let mut arena = arena_new();
    let handles = values
        .iter()
        .map(|v| arena_push(&mut arena, *v).unwrap())
        .collect();
    (arena, handles)
}

#[test]
fn idx_is_copy_eq_ord_hash_without_bounds_on_t() {
    let a: Idx<Opaque> = idx(1);
    let b = a;
    assert_eq!(a, b);
    assert!(idx::<Opaque>(0) < a);
    assert_eq!(HashSet::from([a, b]).len(), 1);
    assert_eq!(format!("{a:?}"), "Idx(1)");
}

#[test]
fn get_out_of_range_is_none() {
    let (arena, _) = filled(&[1, 2]);
    assert_eq!(arena_get(&arena, idx(2)), None);
}

proptest! {
    #[test]
    fn push_then_get_round_trips(values in prop::collection::vec(any::<i64>(), 0..200)) {
        let (arena, handles) = filled(&values);
        for (value, handle) in values.iter().zip(&handles) {
            prop_assert_eq!(arena_get(&arena, *handle), Some(value));
        }
    }

    #[test]
    fn indices_are_sequential_and_len_matches(values in prop::collection::vec(any::<i64>(), 0..200)) {
        let (arena, handles) = filled(&values);
        let raws: Vec<u32> = handles.iter().map(|h| h.raw).collect();
        prop_assert_eq!(raws, (0..values.len() as u32).collect::<Vec<_>>());
        prop_assert_eq!(arena_len(&arena), values.len());
    }

    #[test]
    fn iter_yields_insertion_order(values in prop::collection::vec(any::<i64>(), 0..200)) {
        let (arena, handles) = filled(&values);
        let pairs: Vec<_> = arena_iter(&arena).map(|(h, v)| (h, *v)).collect();
        let expected: Vec<_> = handles.into_iter().zip(values).collect();
        prop_assert_eq!(pairs, expected);
    }

    #[test]
    fn ordering_matches_raw(a in any::<u32>(), b in any::<u32>()) {
        prop_assert_eq!(idx::<i64>(a).cmp(&idx::<i64>(b)), a.cmp(&b));
    }

    #[test]
    fn get_mut_changes_only_target(values in prop::collection::vec(any::<i64>(), 1..50), pick in any::<prop::sample::Index>()) {
        let (mut arena, handles) = filled(&values);
        let target = handles[pick.index(handles.len())];
        *arena_get_mut(&mut arena, target).unwrap() = i64::MIN;
        for (h, v) in handles.iter().zip(&values) {
            let expected = if *h == target { i64::MIN } else { *v };
            prop_assert_eq!(arena_get(&arena, *h), Some(&expected));
        }
    }
}

mod library {
    use crate::arena::{arena_get, arena_len, arena_push};
    use crate::items::{Card, DEFAULT_MIN_SCORE, Question};
    use crate::library::library_new;
    use crate::meta::Meta;
    use crate::stack::Stack;
    use crate::uid::Uid;

    fn meta(uid: u128, title: &str) -> Meta {
        Meta {
            uid: Uid(uid),
            title: title.to_owned(),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: None,
            updated: None,
        }
    }

    #[test]
    fn items_reference_their_stack_and_live_in_separate_arenas() {
        let mut lib = library_new();
        let stack = arena_push(
            &mut lib.stacks,
            Stack {
                meta: meta(1, "French"),
                description: String::new(),
                daily_new_limit: Some(20),
                daily_review_limit: None,
                request_retention: None,
            },
        )
        .unwrap();
        let card = Card {
            meta: meta(2, "Capital of France"),
            stack,
            reversible: false,
            front: "Capital of France?".into(),
            back: "Paris".into(),
        };
        let question = Question {
            meta: meta(3, "Explain FSRS"),
            stack,
            prompt: "Explain FSRS".into(),
            reference: "A memory model".into(),
            accepted: Vec::new(),
            rubric: Vec::new(),
            min_score: DEFAULT_MIN_SCORE,
        };
        let card_idx = arena_push(&mut lib.cards, card.clone()).unwrap();
        arena_push(&mut lib.questions, question).unwrap();

        assert_eq!(arena_get(&lib.cards, card_idx), Some(&card));
        assert_eq!(arena_len(&lib.cards), 1);
        assert_eq!(arena_len(&lib.notes), 0);
        assert_eq!(
            arena_get(&lib.stacks, stack).map(|s| s.meta.title.as_str()),
            Some("French")
        );
    }
}

mod review {
    use proptest::prelude::*;

    use crate::review::{Rating, rating_from_value, rating_value};

    const ALL: [Rating; 4] = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];

    #[test]
    fn value_round_trips_for_every_rating() {
        for rating in ALL {
            assert_eq!(rating_from_value(rating_value(rating)), Some(rating));
        }
    }

    #[test]
    fn ratings_are_ordered_worst_to_best() {
        assert!(ALL.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(ALL.map(rating_value), [1, 2, 3, 4]);
    }

    proptest! {
        #[test]
        fn values_outside_one_to_four_are_rejected(v in any::<u8>()) {
            prop_assert_eq!(rating_from_value(v).is_some(), (1..=4).contains(&v));
        }
    }
}

mod run {
    use std::collections::HashSet;

    use crate::event::{EventKind, event_name, is_item_event};
    use crate::review::Rating;
    use crate::run::{
        RunMode, RunState, run_mode_from_name, run_mode_name, run_state_from_name, run_state_name,
    };

    #[test]
    fn mode_and_state_names_round_trip() {
        for mode in [RunMode::Flip, RunMode::Answer] {
            assert_eq!(run_mode_from_name(run_mode_name(mode)), Some(mode));
        }
        for state in [
            RunState::Idle,
            RunState::Running,
            RunState::Paused,
            RunState::Finished,
            RunState::Aborted,
        ] {
            assert_eq!(run_state_from_name(run_state_name(state)), Some(state));
        }
        assert_eq!(run_state_from_name("bogus"), None);
    }

    #[test]
    fn event_names_are_unique_and_item_events_are_classified() {
        let kinds = [
            EventKind::RunStarted,
            EventKind::ItemShown,
            EventKind::ItemSkipped,
            EventKind::ItemFlipped,
            EventKind::AnswerSubmitted(String::new()),
            EventKind::ItemRated(Rating::Good),
            EventKind::RunPaused,
            EventKind::RunResumed,
            EventKind::RunFinished,
            EventKind::RunAborted,
        ];
        let names: HashSet<_> = kinds.iter().map(event_name).collect();
        assert_eq!(names.len(), kinds.len());
        assert_eq!(kinds.iter().filter(|k| is_item_event(k)).count(), 5);
    }
}
