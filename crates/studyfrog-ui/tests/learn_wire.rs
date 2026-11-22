#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Selecting and deleting a wire on the learn-mode canvas (STORY-0198). Pure update, plus the wire geometry.

use eframe::egui::{Pos2, Rect, vec2};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::learn_editor::{LearnMsg, update};
use studyfrog_ui::modal::Modal;
use studyfrog_ui::state::State;
use studyfrog_ui::view::learn_editor::canvas::{NODE, distance_to_path, wire_path};

fn send(state: State, m: LearnMsg) -> (State, Vec<Effect>) {
    update(state, m)
}

fn editor(state: &State) -> &studyfrog_ui::learn_editor::LearnEditor {
    match state.modal.as_ref().unwrap() {
        Modal::LearnEditor(e) => e,
        other => panic!("not the editor: {other:?}"),
    }
}

/// A mode with three steps and two wires: n1 to n2, then n2 to n3.
fn three_steps_two_wires() -> State {
    let mut state = send(State::default(), LearnMsg::Open).0;
    for _ in 0..3 {
        state = send(state, LearnMsg::AddNode).0;
    }
    for (from, to) in [("n1", "n2"), ("n2", "n3")] {
        state = send(state, LearnMsg::From(from.into())).0;
        state = send(state, LearnMsg::To(to.into())).0;
        state = send(state, LearnMsg::AddWire).0;
    }
    state
}

#[test]
fn clicking_a_wire_selects_it_and_delete_removes_only_that_wire() {
    let state = three_steps_two_wires();
    let (state, _) = send(state, LearnMsg::ClickWire(0));
    assert_eq!(editor(&state).selected_wire, Some(0));
    let (state, fx) = send(state, LearnMsg::DeleteSelected);
    assert!(fx.is_empty());
    let e = editor(&state);
    assert_eq!(e.graph.edges.len(), 1, "one wire goes");
    assert_eq!(e.graph.edges[0].from, "n2", "the other wire is kept");
    assert_eq!(e.graph.nodes.len(), 3, "the steps stay");
    assert_eq!(e.selected_wire, None);
}

#[test]
fn selecting_a_step_clears_the_wire_selection_and_delete_removes_the_step() {
    let state = send(three_steps_two_wires(), LearnMsg::ClickWire(1)).0;
    let state = send(state, LearnMsg::ClickNode("n1".into())).0;
    let e = editor(&state);
    assert_eq!((e.selected.as_deref(), e.selected_wire), (Some("n1"), None));
    let (state, _) = send(state, LearnMsg::DeleteSelected);
    assert_eq!(editor(&state).graph.nodes.len(), 2, "the step went");
    assert_eq!(editor(&state).graph.edges.len(), 1, "its wire went with it");
}

#[test]
fn a_wire_is_not_selected_in_connect_mode_and_removing_a_wire_clears_the_selection() {
    let state = send(three_steps_two_wires(), LearnMsg::ToggleConnect).0;
    let state = send(state, LearnMsg::ClickWire(0)).0;
    assert_eq!(editor(&state).selected_wire, None);
    let state = send(state, LearnMsg::ToggleConnect).0;
    let state = send(state, LearnMsg::ClickWire(1)).0;
    let state = send(state, LearnMsg::RemoveWire(0)).0;
    let e = editor(&state);
    assert_eq!(e.selected_wire, None, "the index of the wire moved");
    assert_eq!(e.graph.edges.len(), 1);
}

#[test]
fn a_click_on_empty_canvas_adds_a_step_and_clears_the_wire_selection() {
    let state = send(three_steps_two_wires(), LearnMsg::ClickWire(0)).0;
    let state = send(state, LearnMsg::PlaceAt(20.0, 20.0)).0;
    let e = editor(&state);
    assert_eq!(e.selected_wire, None);
    assert_eq!(e.graph.nodes.len(), 4);
}

#[test]
fn a_straight_wire_is_hit_near_its_line_and_missed_away_from_it() {
    let a = Rect::from_min_size(Pos2::new(0.0, 0.0), NODE);
    let b = Rect::from_min_size(Pos2::new(400.0, 0.0), NODE);
    let path = wire_path(a, b);
    let mid = Pos2::new((path[0].x + path[1].x) / 2.0, (path[0].y + path[1].y) / 2.0);
    assert!(distance_to_path(&path, mid).unwrap() < 0.5);
    assert!(distance_to_path(&path, mid + vec2(0.0, 40.0)).unwrap() > 30.0);
}

#[test]
fn a_wire_back_to_the_left_is_hit_on_its_arc_above_the_steps() {
    let a = Rect::from_min_size(Pos2::new(400.0, 100.0), NODE);
    let b = Rect::from_min_size(Pos2::new(0.0, 100.0), NODE);
    let path = wire_path(a, b);
    assert_eq!(path.len(), 4, "the arc has its corners");
    let top_run = (path[1] + path[2].to_vec2()) * 0.5;
    assert!(distance_to_path(&path, top_run).unwrap() < 0.5);
    assert!(
        distance_to_path(&path, a.center()).unwrap() > 20.0,
        "not through the steps"
    );
}
