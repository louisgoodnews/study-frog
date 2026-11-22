#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The memory game's entry points: from a calendar session and a planned review (BUG-0046), and
//! from a stack page while a rehearsal run is open (BUG-0047).

mod common;

use common::views::{harness, open_model, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::review_plan::PlanEntry;
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_new, date_to_days};
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::planner_reviews::ReviewMsg;
use studyfrog_ui::setup::memory::{MemoryMsg, MemoryUi};
use studyfrog_ui::state::{Screen, Selection};

fn session() -> Session {
    Session {
        id: Uid(7),
        title: "French drills".into(),
        stack: Some(Uid(1)),
        start: LocalTime {
            date: date_new(2026, 10, 12).unwrap(),
            minute: 9 * 60,
        },
        duration_min: 30,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: Default::default(),
        reminders: Vec::new(),
    }
}

fn calendar_model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Calendar;
    let today = date_new(2026, 10, 12).unwrap();
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Loaded(Ok((vec![session()], today)))),
    );
    m
}

fn review_model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Calendar;
    let today = date_new(2026, 10, 12).unwrap();
    apply(&mut m, Msg::Plan(PlanMsg::Loaded(Ok((Vec::new(), today)))));
    let entry = PlanEntry {
        stack: Some(Uid(1)),
        day: date_to_days(today) + 3,
        items: (0..4).map(|i| Uid(10 + i)).collect(),
        stack_size: 10,
    };
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Reviews(ReviewMsg::Loaded(Ok(vec![entry])))),
    );
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Select(date_new(2026, 10, 15).unwrap())),
    );
    m
}

fn cards(n: usize) -> Vec<(String, String)> {
    (0..n)
        .map(|i| (format!("front {i}"), format!("back {i}")))
        .collect()
}

fn memory(state: &studyfrog_ui::state::State) -> &MemoryUi {
    state.rehearse.memory.as_ref().expect("a memory game")
}

#[test]
fn a_session_starts_a_memory_game_with_its_stack() {
    let mut m = calendar_model();
    let fx = apply(&mut m, Msg::Plan(PlanMsg::Memory(Uid(1))));
    assert_eq!(m.state.screen, Screen::Rehearse);
    assert_eq!(m.state.browser.selection, Selection::Stack("french".into()));
    assert!(memory(&m.state).loading);
    assert!(
        fx.contains(&Effect::MemoryDeck(Some("french".into()))),
        "{fx:?}"
    );
}

#[test]
fn the_memory_game_is_offered_on_a_session_row() {
    let m = calendar_model();
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Memory game for French drills").click();
    h.run_steps(3);
    let s = &h.state().0.state;
    assert_eq!(s.screen, Screen::Rehearse);
    assert!(memory(s).loading);
}

#[test]
fn the_memory_game_is_offered_on_a_planned_review_row() {
    let m = review_model();
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Memory game for French").click();
    h.run_steps(3);
    let s = &h.state().0.state;
    assert_eq!(s.screen, Screen::Rehearse);
    assert_eq!(s.browser.selection, Selection::Stack("french".into()));
    assert!(memory(s).loading);
}

#[test]
fn a_memory_game_takes_the_page_over_an_open_run() {
    let mut m = running_model(RunMode::Flip);
    assert!(m.state.rehearse.run.is_some());
    // The stack page's Memory game button: navigate to Rehearse, then start the game.
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    apply(&mut m, Msg::Memory(MemoryMsg::StartStack("french".into())));
    apply(&mut m, Msg::Memory(MemoryMsg::Loaded(Ok(cards(4)))));
    assert!(memory(&m.state).session.is_some());
    assert!(
        m.state.rehearse.run.is_some(),
        "the run waits, it is not ended"
    );

    let mut h = harness(m);
    h.run_steps(3);
    assert!(h.query_by_label("Close game").is_some(), "the board shows");
    assert!(h.query_by_label("Stop (Esc)").is_none(), "the run does not");
}

#[test]
fn closing_the_memory_game_brings_the_open_run_back() {
    let mut m = running_model(RunMode::Flip);
    apply(&mut m, Msg::Memory(MemoryMsg::StartStack("french".into())));
    apply(&mut m, Msg::Memory(MemoryMsg::Loaded(Ok(cards(4)))));
    apply(&mut m, Msg::Memory(MemoryMsg::Close));
    let mut h = harness(m);
    h.run_steps(3);
    assert!(h.query_by_label("Stop (Esc)").is_some());
    assert!(h.query_by_label("Close game").is_none());
}
