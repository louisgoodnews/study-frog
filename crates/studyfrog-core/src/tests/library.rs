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
