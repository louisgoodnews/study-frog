#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Board invariants over random operation sequences.

use std::collections::HashSet;

use proptest::prelude::*;
use studyfrog_board::edit::{add_card, remove_card};
use studyfrog_board::model::{Board, MoveMode, board_default, card_new, find_card, over_limit};
use studyfrog_board::moves::{move_card, pull, push, shift};
use studyfrog_core::uid::Uid;

const BACKLOG: Uid = Uid(1);

fn board() -> Board {
    board_default([Uid(1), Uid(2), Uid(3), Uid(4)])
}

#[derive(Debug, Clone)]
enum Op {
    Add(usize),
    Move(usize, usize, usize),
    Push(usize),
    Pull(usize),
    Shift(usize, i32),
    Remove(usize),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0_usize..4).prop_map(Op::Add),
        (any::<usize>(), 0_usize..4, 0_usize..6).prop_map(|(c, t, i)| Op::Move(c, t, i)),
        any::<usize>().prop_map(Op::Push),
        any::<usize>().prop_map(Op::Pull),
        (any::<usize>(), -3_i32..4).prop_map(|(c, d)| Op::Shift(c, d)),
        any::<usize>().prop_map(Op::Remove),
    ]
}

fn ids(b: &Board) -> Vec<Uid> {
    b.columns
        .iter()
        .flat_map(|c| c.cards.iter().map(|k| k.id))
        .collect()
}

proptest! {
    #[test]
    fn strict_operations_keep_every_invariant(ops in prop::collection::vec(op(), 0..80)) {
        let mut b = board();
        let (mut next_id, mut expected) = (1000_u128, 0_usize);
        for o in ops {
            let all = ids(&b);
            let pick = |n: usize| all.get(n % all.len().max(1)).copied().unwrap_or(Uid(0));
            let col = |i: usize| b.columns[i].id;
            let result = match o {
                Op::Add(c) => { next_id += 1; add_card(&b, col(c), card_new(Uid(next_id), "n"), MoveMode::Strict) }
                Op::Move(c, t, i) => move_card(&b, pick(c), col(t), i, MoveMode::Strict),
                Op::Push(c) => push(&b, pick(c), MoveMode::Strict),
                Op::Pull(c) => pull(&b, pick(c), MoveMode::Strict),
                Op::Shift(c, d) => shift(&b, pick(c), d),
                Op::Remove(c) => remove_card(&b, pick(c)),
            };
            match (&o, result) {
                (Op::Add(_), Ok(n)) => { expected += 1; b = n; }
                (Op::Remove(_), Ok(n)) => { expected -= 1; b = n; }
                (_, Ok(n)) => b = n,
                (_, Err(_)) => {} // a refused operation leaves `b` untouched by construction
            }
            let all = ids(&b);
            prop_assert_eq!(all.len(), expected);
            prop_assert_eq!(all.iter().collect::<HashSet<_>>().len(), all.len(), "ids stay unique");
            prop_assert!(over_limit(&b).is_empty(), "strict mode never exceeds a limit");
        }
    }

    #[test]
    fn a_move_followed_by_the_reverse_move_restores_the_board(n in 1_u128..6, pick in 0_usize..6) {
        let b = (0..n).fold(board(), |b, i| add_card(&b, BACKLOG, card_new(Uid(100 + i), "t"), MoveMode::Strict).unwrap());
        let id = Uid(100 + (pick as u128 % n));
        let (c, i) = find_card(&b, id).unwrap();
        let moved = push(&b, id, MoveMode::Strict).unwrap();
        let back = move_card(&moved, id, b.columns[c].id, i, MoveMode::Strict).unwrap();
        prop_assert_eq!(back, b);
    }
}
