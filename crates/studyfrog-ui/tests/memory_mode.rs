#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Memory mode: the grid's shape and the game's state (start, load, flip, clock, close).

use studyfrog_ui::effect::Effect;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::memory::{MemoryMsg, MemoryUi};
use studyfrog_ui::setup::memory_layout::{CARD_ASPECT, GAP, grid_for, grid_height, grid_width};
use studyfrog_ui::state::{Selection, State};
use studyfrog_ui::update::update;

fn cards(n: usize) -> Vec<(String, String)> {
    (0..n)
        .map(|i| (format!("front {i}"), format!("back {i}")))
        .collect()
}

fn send(state: State, m: MemoryMsg) -> (State, Vec<Effect>) {
    update(state, Msg::Memory(m))
}

fn memory(state: &State) -> &MemoryUi {
    state.rehearse.memory.as_ref().expect("a memory game")
}

#[test]
fn grid_fits_the_space_at_card_aspect() {
    for tiles in [12, 16, 20] {
        let g = grid_for(tiles, 800.0, 600.0).expect("a grid");
        assert!(grid_width(&g) <= 800.0 + 0.01, "{tiles}: {g:?}");
        assert!(grid_height(&g) <= 600.0 + 0.01, "{tiles}: {g:?}");
        assert!((g.tile_h - g.tile_w * CARD_ASPECT).abs() < 0.01);
        assert!(g.cols * g.rows >= tiles);
        assert!(g.tile_w > 0.0 && GAP > 0.0);
    }
}

#[test]
fn grid_is_none_without_cards_or_room() {
    assert!(grid_for(0, 800.0, 600.0).is_none());
    assert!(grid_for(12, 0.0, 600.0).is_none());
    assert!(grid_for(12, 800.0, -1.0).is_none());
}

#[test]
fn grid_gets_larger_with_more_room() {
    let small = grid_for(16, 400.0, 400.0).expect("small");
    let large = grid_for(16, 1200.0, 900.0).expect("large");
    assert!(large.tile_w > small.tile_w);
}

#[test]
fn start_without_a_stack_explains_why() {
    let state = State::default();
    let (state, effects) = send(state, MemoryMsg::Start);
    assert!(effects.is_empty());
    assert!(memory(&state).error.is_some());
    assert!(memory(&state).session.is_none());
}

#[test]
fn start_in_a_stack_reads_its_cards() {
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, effects) = send(state, MemoryMsg::Start);
    assert!(memory(&state).loading);
    assert_eq!(effects, vec![Effect::MemoryDeck(Some("bio".to_owned()))]);
}

#[test]
fn loaded_cards_make_a_game_of_up_to_eight_pairs() {
    let mut state = State::default();
    state.browser.selection = Selection::Inbox;
    let (state, _) = send(state, MemoryMsg::Start);
    let (state, _) = send(state, MemoryMsg::Loaded(Ok(cards(20))));
    let session = memory(&state).session.clone().expect("a game");
    assert_eq!(session.tiles.len(), 16);
    assert!(!memory(&state).loading);
}

#[test]
fn too_few_cards_give_an_error() {
    let mut state = State::default();
    state.browser.selection = Selection::Inbox;
    let (state, _) = send(state, MemoryMsg::Start);
    let (state, _) = send(state, MemoryMsg::Loaded(Ok(cards(1))));
    assert!(memory(&state).session.is_none());
    assert!(memory(&state).error.is_some());
}

#[test]
fn a_mismatch_hides_again_after_the_delay() {
    let mut state = State::default();
    state.browser.selection = Selection::Inbox;
    let (state, _) = send(state, MemoryMsg::Start);
    let (mut state, _) = send(state, MemoryMsg::Loaded(Ok(cards(4))));
    state.wall_ms = memory(&state).started_ms + 10;
    // Find two tiles of different pairs.
    let session = memory(&state).session.clone().expect("a game");
    let first = 0;
    let second = session
        .tiles
        .iter()
        .position(|t| t.pair != session.tiles[first].pair)
        .expect("another pair");
    let (state, _) = send(state, MemoryMsg::Flip(first));
    let (state, _) = send(state, MemoryMsg::Flip(second));
    assert!(
        memory(&state)
            .session
            .as_ref()
            .expect("game")
            .mismatch
            .is_some()
    );
    let mut state = state;
    state.wall_ms = memory(&state).started_ms + 2_000;
    let (state, _) = send(state, MemoryMsg::Clock);
    assert!(
        memory(&state)
            .session
            .as_ref()
            .expect("game")
            .mismatch
            .is_none()
    );
}

#[test]
fn close_removes_the_game() {
    let mut state = State::default();
    state.browser.selection = Selection::Inbox;
    let (state, _) = send(state, MemoryMsg::Start);
    let (state, _) = send(state, MemoryMsg::Close);
    assert!(state.rehearse.memory.is_none());
}
