#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The memory grid on screen (STORY-0174: keyboard play and accessible names) and the two launch
//! points of a game (STORY-0175: the rehearse setup and the stack page).

mod common;

use common::views::{App, harness, open_model};
use eframe::egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use studyfrog_rehearsal::memory_deck::FlipState;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::memory::MemoryMsg;
use studyfrog_ui::state::{Screen, Selection};

/// A model with a game of `pairs` pairs started and its cards loaded, on the rehearse screen.
fn started(pairs: usize) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.browser.selection = Selection::Stack("s".to_owned());
    apply(&mut m, Msg::Memory(MemoryMsg::Start));
    let cards = (0..pairs)
        .map(|i| (format!("front {i}"), format!("back {i}")))
        .collect();
    apply(&mut m, Msg::Memory(MemoryMsg::Loaded(Ok(cards))));
    m
}

fn card_states(h: &Harness<'_, App>) -> Vec<FlipState> {
    let memory = h.state().0.state.rehearse.memory.as_ref().expect("a game");
    memory
        .session
        .as_ref()
        .expect("a session")
        .tiles
        .iter()
        .map(|t| t.state)
        .collect()
}

#[test]
fn a_card_turns_over_from_the_keyboard() {
    let mut h = harness(started(2));
    h.run_steps(2);
    h.get_by_label("Card 1, face down").focus();
    h.run_steps(1);
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(card_states(&h)[0], FlipState::FaceUp);
}

#[test]
fn the_arrow_keys_move_to_the_next_card() {
    let mut h = harness(started(4));
    h.run_steps(2);
    h.get_by_label("Card 1, face down").focus();
    h.run_steps(1);
    h.key_press(Key::ArrowRight);
    h.run_steps(1);
    h.key_press(Key::Space);
    h.run_steps(3);
    let states = card_states(&h);
    assert_eq!(states[0], FlipState::FaceDown);
    assert_eq!(states[1], FlipState::FaceUp);
}

#[test]
fn every_card_has_an_accessible_name_that_says_its_state() {
    let mut h = harness(started(2));
    h.run_steps(2);
    assert!(h.query_by_label("Card 1, face down").is_some());
    assert!(h.query_by_label("Card 4, face down").is_some());
    h.get_by_label("Card 1, face down").click();
    h.run_steps(3);
    assert!(h.query_all_by_label_contains("Card ").count() >= 4);
}

#[test]
fn the_rehearse_setup_starts_a_memory_game() {
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.browser.selection = Selection::Stack("french".to_owned());
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Memory game").click();
    h.run_steps(2);
    assert!(h.state().0.state.rehearse.memory.is_some());
}

#[test]
fn the_stack_page_starts_a_memory_game_in_rehearse() {
    let mut m = open_model();
    m.state.browser.selection = Selection::Stack("french".to_owned());
    let mut h = harness(m);
    h.run_steps(2);
    // The stack page keeps Memory game in More actions, beside the less common actions.
    h.get_by_label("More actions").click();
    h.run_steps(2);
    h.get_by_label("Memory game").click();
    h.run_steps(2);
    let state = &h.state().0.state;
    assert_eq!(state.screen, Screen::Rehearse);
    assert!(state.rehearse.memory.is_some());
}
