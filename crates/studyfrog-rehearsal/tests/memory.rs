#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for Memory Mode: deck pairing, deterministic shuffle, flip cycle, mismatch timing, and the
//! time limit.

use studyfrog_rehearsal::memory::{
    FlipError, MemoryOutcome, flip_tile, new_memory_session, tick_memory_timer,
};
use studyfrog_rehearsal::memory_deck::{FlipState, TileRole, build_tiles};

fn cards(n: usize) -> Vec<(String, String)> {
    (0..n).map(|i| (format!("q{i}"), format!("a{i}"))).collect()
}

fn index_of(
    tiles: &[studyfrog_rehearsal::memory_deck::MemoryTile],
    pair: u32,
    role: TileRole,
) -> usize {
    tiles
        .iter()
        .find(|t| t.pair == pair && t.role == role)
        .unwrap()
        .index
}

#[test]
fn each_card_gives_two_tiles_sharing_a_pair_id() {
    let tiles = build_tiles(&cards(6), 6, 1);
    assert_eq!(tiles.len(), 12);
    for pair in 0..6u32 {
        let members: Vec<_> = tiles.iter().filter(|t| t.pair == pair).collect();
        assert_eq!(members.len(), 2);
        assert_ne!(members[0].role, members[1].role);
    }
}

#[test]
fn the_same_seed_gives_the_same_layout_and_a_different_seed_differs() {
    let a = build_tiles(&cards(8), 8, 42);
    let b = build_tiles(&cards(8), 8, 42);
    let c = build_tiles(&cards(8), 8, 43);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn pairs_limit_and_blank_cards_are_skipped() {
    let mut deck = cards(5);
    deck[1].1 = "  ".to_owned();
    let tiles = build_tiles(&deck, 3, 7);
    assert_eq!(tiles.len(), 6);
    assert!(tiles.iter().all(|t| t.pair != 1));
    assert!(build_tiles(&deck, 0, 7).is_empty());
}

#[test]
fn matching_two_tiles_removes_them_and_extends_the_streak() {
    let s = new_memory_session(&cards(2), 2, 3, 1000, None);
    let front = index_of(&s.tiles, 0, TileRole::Front);
    let back = index_of(&s.tiles, 0, TileRole::Back);
    let s = flip_tile(&s, front).unwrap();
    let s = flip_tile(&s, back).unwrap();
    assert_eq!(s.tiles[front].state, FlipState::Matched);
    assert_eq!(s.moves, 1);
    assert_eq!(s.streak, 1);
    assert_eq!(s.best_streak, 1);
    assert!(s.outcome.is_none());
}

#[test]
fn a_mismatch_blocks_flips_until_the_delay_has_passed() {
    let s = new_memory_session(&cards(2), 2, 3, 1000, None);
    let a = index_of(&s.tiles, 0, TileRole::Front);
    let b = index_of(&s.tiles, 1, TileRole::Front);
    let c = index_of(&s.tiles, 1, TileRole::Back);
    let s = flip_tile(&s, a).unwrap();
    let s = flip_tile(&s, b).unwrap();
    assert_eq!(s.tiles[a].state, FlipState::MismatchWait);
    assert_eq!(s.streak, 0);
    assert_eq!(flip_tile(&s, c), Err(FlipError::Blocked));
    let early = tick_memory_timer(&s, 999);
    assert_eq!(early.tiles[a].state, FlipState::MismatchWait);
    let later = tick_memory_timer(&early, 1000);
    assert_eq!(later.tiles[a].state, FlipState::FaceDown);
    assert!(flip_tile(&later, c).is_ok());
}

#[test]
fn flipping_a_face_up_tile_is_refused() {
    let s = new_memory_session(&cards(2), 2, 3, 1000, None);
    let a = index_of(&s.tiles, 0, TileRole::Front);
    let s = flip_tile(&s, a).unwrap();
    assert_eq!(flip_tile(&s, a), Err(FlipError::NotFaceDown));
    assert_eq!(flip_tile(&s, 99), Err(FlipError::NoSuchTile));
}

#[test]
fn clearing_every_pair_completes_the_session_with_its_time() {
    let mut s = new_memory_session(&cards(2), 2, 9, 1000, None);
    s = tick_memory_timer(&s, 4200);
    for pair in 0..2u32 {
        let f = index_of(&s.tiles, pair, TileRole::Front);
        let b = index_of(&s.tiles, pair, TileRole::Back);
        s = flip_tile(&s, f).unwrap();
        s = flip_tile(&s, b).unwrap();
    }
    assert_eq!(s.best_streak, 2);
    assert_eq!(
        s.outcome,
        Some(MemoryOutcome::Completed { elapsed_ms: 4200 })
    );
    assert_eq!(flip_tile(&s, 0), Err(FlipError::Finished));
}

#[test]
fn the_time_limit_ends_the_session_and_time_never_runs_backwards() {
    let s = new_memory_session(&cards(3), 3, 1, 1000, Some(60_000));
    let s = tick_memory_timer(&s, 59_999);
    assert!(s.outcome.is_none());
    let s = tick_memory_timer(&s, 60_000);
    assert_eq!(s.outcome, Some(MemoryOutcome::TimedOut));
    let back = tick_memory_timer(&s, 10);
    assert_eq!(back.elapsed_ms, 60_000);
}
