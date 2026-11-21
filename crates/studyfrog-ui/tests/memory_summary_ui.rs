#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The memory game's end on screen (STORY-0175): the victory summary and the timeout line.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_rehearsal::memory_summary::MemoryTimer;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::memory::MemoryMsg;
use studyfrog_ui::state::{Screen, Selection};

fn cards() -> Vec<(String, String)> {
    (0..2)
        .map(|i| (format!("front {i}"), format!("back {i}")))
        .collect()
}

fn started(timer: MemoryTimer) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.browser.selection = Selection::Stack("s".to_owned());
    m.state.rehearse.memory_timer = timer;
    apply(&mut m, Msg::Memory(MemoryMsg::Start));
    apply(&mut m, Msg::Memory(MemoryMsg::Loaded(Ok(cards()))));
    m
}

fn tick(m: &mut Model, ms: i64) {
    let start = m.state.rehearse.memory.as_ref().expect("a game").started_ms;
    m.state.wall_ms = start + ms;
    apply(m, Msg::Memory(MemoryMsg::Clock));
}

#[test]
fn a_finished_game_shows_the_summary_and_the_first_record() {
    let mut m = started(MemoryTimer::Stopwatch);
    tick(&mut m, 30_000);
    let tiles = m
        .state
        .rehearse
        .memory
        .as_ref()
        .expect("a game")
        .session
        .clone()
        .expect("a session")
        .tiles;
    for pair in 0..2u32 {
        for (i, _) in tiles.iter().enumerate().filter(|(_, t)| t.pair == pair) {
            apply(&mut m, Msg::Memory(MemoryMsg::Flip(i)));
        }
    }
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Memory game complete").is_some());
    assert!(
        h.query_all_by_label_contains("All pairs found in 0:30.")
            .count()
            >= 1
    );
    assert!(h.query_by_label("Your first time on record.").is_some());
    assert!(h.query_all_by_label("Play again").count() >= 1);
}

#[test]
fn a_countdown_that_runs_out_shows_the_encouraging_line() {
    let mut m = started(MemoryTimer::TimeAttack(60));
    tick(&mut m, 61_000);
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_by_label("Time is up. Here are the cards: try again and beat your time.")
            .is_some()
    );
}
