#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The memory game from the rehearse setup: it plays the stack ticked in the What step (BUG-0024),
//! names the stack when it has too few cards, replays its own stack, and a timeout shows the cards.

mod common;

use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::memory_summary::MemoryTimer;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::memory::{MemoryMsg, MemoryUi};
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

/// A state whose vault has stacks `bio` (uid 1) and `diag` (uid 2), with `diag` ticked.
fn with_diag_ticked() -> State {
    let mut state = State {
        vault: Some(common::views::vault_with_stacks(&[(1, "bio"), (2, "diag")])),
        ..State::default()
    };
    state.rehearse.form.stacks = vec![Uid(2)];
    state
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

#[test]
fn the_memory_game_plays_the_stack_ticked_in_the_setup() {
    let mut state = with_diag_ticked();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (_, effects) = send(state, MemoryMsg::Start);
    assert!(
        effects.contains(&Effect::MemoryDeck(Some("diag".to_owned()))),
        "{effects:?}"
    );
}

#[test]
fn the_stack_page_game_still_plays_its_own_stack_over_a_ticked_one() {
    let (_, effects) = send(with_diag_ticked(), MemoryMsg::StartStack("bio".to_owned()));
    assert!(
        effects.contains(&Effect::MemoryDeck(Some("bio".to_owned()))),
        "{effects:?}"
    );
}

#[test]
fn too_few_cards_names_the_stack_that_was_played() {
    let (state, _) = send(with_diag_ticked(), MemoryMsg::Start);
    let (state, _) = send(state, MemoryMsg::Loaded(Ok(cards(1))));
    let error = memory(&state).error.clone().unwrap_or_default();
    assert!(error.contains("diag"), "{error}");
}

#[test]
fn play_again_replays_the_same_stack_not_the_current_pick() {
    let (state, _) = send(with_diag_ticked(), MemoryMsg::Start);
    let (mut state, _) = send(state, MemoryMsg::Loaded(Ok(cards(4))));
    state.rehearse.form.stacks = Vec::new();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (_, effects) = send(state, MemoryMsg::Again);
    assert!(
        effects.contains(&Effect::MemoryDeck(Some("diag".to_owned()))),
        "{effects:?}"
    );
}

#[test]
fn a_time_attack_timeout_reveals_the_unmatched_cards() {
    use studyfrog_rehearsal::memory_deck::FlipState;
    let mut state = started_on("s", MemoryTimer::TimeAttack(60), 4);
    state.wall_ms += 61_000;
    let (state, _) = send(state, MemoryMsg::Clock);
    let session = memory(&state).session.clone().expect("a game");
    assert!(session.tiles.iter().all(|t| t.state != FlipState::FaceDown));
}
