#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Memory mode: the grid's shape and the game's state (start, load, flip, clock, close).

use studyfrog_core::settings::{memory_best_of, with_memory_best};
use studyfrog_rehearsal::memory::MemoryOutcome;
use studyfrog_rehearsal::memory_summary::MemoryTimer;
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

fn started_on(folder: &str, timer: MemoryTimer, n: usize) -> State {
    let mut state = State::default();
    state.rehearse.memory_timer = timer;
    state.browser.selection = Selection::Stack(folder.to_owned());
    let (state, _) = send(state, MemoryMsg::Start);
    let (mut state, _) = send(state, MemoryMsg::Loaded(Ok(cards(n))));
    state.wall_ms = memory(&state).started_ms;
    state
}

/// Flip every pair in turn, so the game completes at the clock's current time.
fn finish_all(mut state: State) -> (State, Vec<Effect>) {
    let mut effects = Vec::new();
    let session = memory(&state).session.clone().expect("a game");
    let pairs: std::collections::BTreeSet<u32> = session.tiles.iter().map(|t| t.pair).collect();
    for pair in pairs {
        let idx: Vec<usize> = session
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.pair == pair)
            .map(|(i, _)| i)
            .collect();
        for i in idx {
            let (next, fx) = send(state, MemoryMsg::Flip(i));
            state = next;
            effects.extend(fx);
        }
    }
    (state, effects)
}

#[test]
fn a_countdown_ends_in_a_timeout_after_its_limit() {
    let mut state = started_on("s", MemoryTimer::TimeAttack(60), 4);
    state.wall_ms += 61_000;
    let (state, _) = send(state, MemoryMsg::Clock);
    let session = memory(&state).session.clone().expect("a game");
    assert_eq!(session.outcome, Some(MemoryOutcome::TimedOut));
}

#[test]
fn a_finished_game_saves_its_time_as_the_stacks_best() {
    let mut state = started_on("s", MemoryTimer::Stopwatch, 2);
    state.app.vault = Some("/vault".to_owned());
    state.wall_ms += 30_000;
    let (state, _) = send(state, MemoryMsg::Clock);
    let (state, effects) = finish_all(state);
    assert_eq!(memory_best_of(&state.app, "/vault", "s"), Some(30_000));
    assert!(
        matches!(effects.as_slice(), [Effect::SaveAppSettings(_)]),
        "{effects:?}"
    );
}

#[test]
fn a_faster_game_lowers_the_best_and_a_slower_one_does_not_raise_it() {
    let mut state = State::default();
    state.app.vault = Some("/vault".to_owned());
    for (ms, expected) in [(30_000, 30_000), (20_000, 20_000), (50_000, 20_000)] {
        let mut game = started_on("s", MemoryTimer::Stopwatch, 2);
        game.app = state.app.clone();
        game.wall_ms += ms;
        let (game, _) = send(game, MemoryMsg::Clock);
        let (game, _) = finish_all(game);
        assert_eq!(memory_best_of(&game.app, "/vault", "s"), Some(expected));
        state = game;
    }
}

#[test]
fn the_prior_best_is_read_when_a_game_starts() {
    let mut state = State::default();
    state.app.vault = Some("/vault".to_owned());
    state.app = with_memory_best(&state.app, "/vault", "s", 12_000);
    let game = started_on("s", MemoryTimer::Stopwatch, 2);
    let mut game = game;
    game.app = state.app.clone();
    game.browser.selection = Selection::Stack("s".to_owned());
    let (game, _) = send(game, MemoryMsg::Start);
    assert_eq!(memory(&game).prior_best_ms, Some(12_000));
}

#[test]
fn the_chosen_timing_is_used_by_the_next_game() {
    let state = State::default();
    let (state, _) = send(state, MemoryMsg::Mode(MemoryTimer::TimeAttack(90)));
    assert_eq!(state.rehearse.memory_timer, MemoryTimer::TimeAttack(90));
    let mut state = state;
    state.browser.selection = Selection::Inbox;
    let (state, _) = send(state, MemoryMsg::Start);
    let (state, _) = send(state, MemoryMsg::Loaded(Ok(cards(4))));
    assert_eq!(memory(&state).timer, MemoryTimer::TimeAttack(90));
    let session = memory(&state).session.clone().expect("a game");
    assert_eq!(session.limit_ms, Some(90_000));
}

#[test]
fn the_stack_page_starts_a_game_from_its_stack() {
    let state = State::default();
    let (state, effects) = send(state, MemoryMsg::StartStack("bio".to_owned()));
    assert_eq!(effects, vec![Effect::MemoryDeck(Some("bio".to_owned()))]);
    assert_eq!(state.browser.selection, Selection::Stack("bio".to_owned()));
}
