#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The math drill (STORY-0184): opening a seeded problem, checking working and the answer, the keypad,
//! and the full solution.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_rehearsal::formula_gen::generate_formula_exercise;
use studyfrog_rehearsal::formula_trace::{Mistake, se_trace};
use studyfrog_ui::setup::math::{MathMsg, update};
use studyfrog_ui::state::{Screen, State};

fn send(state: State, m: MathMsg) -> State {
    update(state, m).0
}

fn opened(seed: u64) -> State {
    let state = State {
        wall_ms: seed as i64,
        ..State::default()
    };
    send(state, MathMsg::Open)
}

fn drill(state: &State) -> &studyfrog_ui::setup::math::MathDrill {
    state.rehearse.math.as_ref().expect("the drill")
}

#[test]
fn opening_gives_the_problem_for_its_seed() {
    let state = opened(42);
    assert_eq!(drill(&state).exercise, generate_formula_exercise(42));
    assert!(drill(&state).steps.iter().all(String::is_empty));
}

#[test]
fn correct_working_and_answer_are_accepted() {
    let state = opened(7);
    let trace = se_trace(drill(&state).exercise.s_x, drill(&state).exercise.r_xx);
    let mut state = state;
    for (i, step) in trace.iter().enumerate() {
        state = send(state, MathMsg::Step(i, format!("{}", step.value)));
    }
    state = send(state, MathMsg::Answer(format!("{}", trace[2].value)));
    state = send(state, MathMsg::Check);
    let f = drill(&state).feedback.clone().expect("feedback");
    assert!(f.answer_correct);
    assert_eq!(f.diagnosis.deviates_at, None);
}

#[test]
fn a_forgotten_root_is_named_at_its_step() {
    let state = opened(7);
    let d = drill(&state).exercise.clone();
    let trace = se_trace(d.s_x, d.r_xx);
    let mut state = send(state, MathMsg::Step(0, format!("{}", trace[0].value)));
    state = send(state, MathMsg::Step(1, format!("{}", trace[0].value)));
    state = send(state, MathMsg::Check);
    let f = drill(&state).feedback.clone().expect("feedback");
    assert_eq!(f.diagnosis.deviates_at, Some(1));
    assert_eq!(f.diagnosis.mistake, Some(Mistake::ForgotRoot));
    assert!(!f.answer_correct);
}

#[test]
fn the_keypad_types_into_the_active_row_and_deletes() {
    let mut state = opened(3);
    state = send(state, MathMsg::Active(2));
    for k in ["1", ".", "5"] {
        state = send(state, MathMsg::Key(k.to_owned()));
    }
    assert_eq!(drill(&state).steps[2], "1.5");
    state = send(state, MathMsg::Key("Del".to_owned()));
    assert_eq!(drill(&state).steps[2], "1.");
    state = send(state, MathMsg::Key("C".to_owned()));
    assert!(drill(&state).steps[2].is_empty());
}

#[test]
fn the_solution_is_shown_on_demand_and_hidden_again() {
    let state = opened(9);
    let state = send(state, MathMsg::Solution);
    assert!(drill(&state).solution);
    let state = send(state, MathMsg::Solution);
    assert!(!drill(&state).solution);
}

#[test]
fn the_view_shows_the_formula_the_given_values_and_the_check() {
    let state = opened(11);
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.rehearse = state.rehearse;
    let mut h = harness(m);
    h.run_steps(4);
    for text in ["s_e = s_x * sqrt(1 - r_xx)", "Check", "Show full solution"] {
        assert!(h.query_by_label_contains(text).is_some(), "missing {text}");
    }
}
