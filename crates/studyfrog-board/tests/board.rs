#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Board rules: WIP limits, drag/push/pull, editing, and invariants over random operations.

use studyfrog_board::edit::{
    add_card, add_column, edit_column, move_column, remove_card, remove_column, update_card,
};
use studyfrog_board::model::{
    Board, BoardError, MoveMode, board_default, card_count, card_new, find_card, over_limit,
};
use studyfrog_board::moves::{move_card, pull, push, shift};
use studyfrog_core::uid::Uid;

const BACKLOG: Uid = Uid(1);
const DOING: Uid = Uid(2);
const REVIEW: Uid = Uid(3);
const DONE: Uid = Uid(4);

fn board() -> Board {
    board_default([BACKLOG, DOING, REVIEW, DONE])
}

fn with_cards(n: u128) -> Board {
    (0..n).fold(board(), |b, i| {
        add_card(
            &b,
            BACKLOG,
            card_new(Uid(100 + i), &format!("t{i}")),
            MoveMode::Strict,
        )
        .unwrap()
    })
}

fn titles(b: &Board, col: usize) -> Vec<&str> {
    b.columns[col]
        .cards
        .iter()
        .map(|c| c.title.as_str())
        .collect()
}

#[test]
fn strict_moves_respect_the_limit_and_force_does_not() {
    let mut b = with_cards(5);
    for i in 0..3 {
        b = push(&b, Uid(100 + i), MoveMode::Strict).unwrap();
    }
    assert_eq!(
        titles(&b, 1),
        ["t2", "t1", "t0"],
        "pushed cards go to the top"
    );
    let refused = push(&b, Uid(103), MoveMode::Strict);
    assert_eq!(
        refused,
        Err(BoardError::WipLimit {
            column: DOING,
            limit: 3
        })
    );
    assert_eq!(card_count(&b), 5);

    let forced = push(&b, Uid(103), MoveMode::Force).unwrap();
    assert_eq!(
        over_limit(&forced),
        [DOING],
        "a forced move is visible as over the limit"
    );
    assert_eq!(titles(&forced, 1).len(), 4);
}

#[test]
fn reordering_inside_a_full_column_is_always_allowed() {
    let mut b = with_cards(3);
    for i in 0..3 {
        b = push(&b, Uid(100 + i), MoveMode::Strict).unwrap();
    }
    let moved = move_card(&b, Uid(100), DOING, 0, MoveMode::Strict).unwrap();
    assert_eq!(titles(&moved, 1), ["t0", "t2", "t1"]);
    assert_eq!(
        titles(&shift(&moved, Uid(100), 1).unwrap(), 1),
        ["t2", "t0", "t1"]
    );
    assert_eq!(
        titles(&shift(&moved, Uid(100), -5).unwrap(), 1),
        ["t0", "t2", "t1"],
        "shifting clamps at the top"
    );
}

#[test]
fn push_and_pull_walk_a_card_across_the_board() {
    let b = with_cards(1);
    let id = Uid(100);
    let mut b = push(&b, id, MoveMode::Strict).unwrap();
    assert_eq!(find_card(&b, id), Some((1, 0)));
    b = push(&b, id, MoveMode::Strict).unwrap();
    b = push(&b, id, MoveMode::Strict).unwrap();
    assert_eq!(find_card(&b, id), Some((3, 0)));
    assert_eq!(
        push(&b, id, MoveMode::Strict),
        Err(BoardError::AtLastColumn)
    );
    b = pull(&b, id, MoveMode::Strict).unwrap();
    assert_eq!(find_card(&b, id), Some((2, 0)));
    let backlog = with_cards(1);
    assert_eq!(
        pull(&backlog, id, MoveMode::Strict),
        Err(BoardError::AtFirstColumn)
    );
}

#[test]
fn unknown_ids_are_reported_and_leave_the_board_alone() {
    let b = with_cards(2);
    assert_eq!(
        move_card(&b, Uid(999), DOING, 0, MoveMode::Strict),
        Err(BoardError::NoSuchCard(Uid(999)))
    );
    assert_eq!(
        move_card(&b, Uid(100), Uid(999), 0, MoveMode::Strict),
        Err(BoardError::NoSuchColumn(Uid(999)))
    );
    assert_eq!(
        add_card(&b, BACKLOG, card_new(Uid(100), "dup"), MoveMode::Strict),
        Err(BoardError::DuplicateId(Uid(100)))
    );
    assert_eq!(
        add_card(
            &b,
            BACKLOG,
            card_new(Uid(1), "clashes with a column"),
            MoveMode::Strict
        ),
        Err(BoardError::DuplicateId(BACKLOG))
    );
    assert_eq!(
        remove_card(&b, Uid(999)),
        Err(BoardError::NoSuchCard(Uid(999)))
    );
}

#[test]
fn adding_to_a_full_column_follows_the_mode() {
    let mut b = board();
    for i in 0..3 {
        b = add_card(&b, DOING, card_new(Uid(10 + i), "x"), MoveMode::Strict).unwrap();
    }
    assert!(matches!(
        add_card(&b, DOING, card_new(Uid(20), "y"), MoveMode::Strict),
        Err(BoardError::WipLimit { .. })
    ));
    assert_eq!(
        over_limit(&add_card(&b, DOING, card_new(Uid(20), "y"), MoveMode::Force).unwrap()),
        [DOING]
    );
}

#[test]
fn editing_cards_keeps_their_place_and_id() {
    let b = with_cards(3);
    let b = update_card(&b, Uid(101), |c| {
        c.title = "renamed".into();
        c.points = Some(5);
        c.stack = Some(Uid(7));
        c.id = Uid(555); // an attempt to change the identity is ignored
    })
    .unwrap();
    assert_eq!(find_card(&b, Uid(101)), Some((0, 1)));
    let card = &b.columns[0].cards[1];
    assert_eq!(
        (card.title.as_str(), card.points, card.stack),
        ("renamed", Some(5), Some(Uid(7)))
    );
    assert_eq!(card_count(&remove_card(&b, Uid(101)).unwrap()), 2);
}

#[test]
fn column_editing_never_loses_cards() {
    let mut b = with_cards(4);
    b = push(&b, Uid(100), MoveMode::Strict).unwrap();
    b = add_column(&b, Uid(9), "Blocked", Some(1), 2).unwrap();
    assert_eq!(
        b.columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["Backlog", "Doing", "Blocked", "Review", "Done"]
    );
    b = edit_column(&b, DOING, "In progress", Some(0)).unwrap();
    assert_eq!(
        over_limit(&b),
        [DOING],
        "lowering a limit below the count flags the column, it does not move cards"
    );
    b = remove_column(&b, DOING).unwrap();
    assert_eq!(card_count(&b), 4);
    assert_eq!(
        find_card(&b, Uid(100)).map(|p| p.0),
        Some(0),
        "cards of a removed column go to the previous one"
    );
    b = move_column(&b, DONE, 0).unwrap();
    assert_eq!(b.columns[0].id, DONE);
    assert_eq!(
        remove_column(
            &Board {
                columns: vec![b.columns[0].clone()]
            },
            DONE
        ),
        Err(BoardError::LastRemainingColumn)
    );
    let first = remove_column(&with_cards(2), BACKLOG).unwrap();
    assert_eq!(
        card_count(&first),
        2,
        "removing the first column hands its cards to the next"
    );
}
