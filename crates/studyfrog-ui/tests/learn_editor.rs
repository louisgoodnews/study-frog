#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The learn-mode editor (STORY-0198): it lists and opens modes through effects, builds a graph from messages,
//! refuses to save a graph with reasons, and removing a node takes its wires with it.

use studyfrog_rehearsal::learn_graph::NodeKind;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::learn_editor::{LearnEffect, LearnMsg, update};
use studyfrog_ui::modal::Modal;
use studyfrog_ui::state::State;

fn send(state: State, m: LearnMsg) -> (State, Vec<Effect>) {
    update(state, m)
}

fn editor(state: &State) -> &studyfrog_ui::learn_editor::LearnEditor {
    match state.modal.as_ref().unwrap() {
        Modal::LearnEditor(e) => e,
        other => panic!("not the editor: {other:?}"),
    }
}

#[test]
fn opening_the_editor_lists_the_stored_modes() {
    let (state, fx) = send(State::default(), LearnMsg::Open);
    assert!(matches!(state.modal, Some(Modal::LearnEditor(_))));
    assert_eq!(fx, vec![Effect::Learn(LearnEffect::List)]);
    assert!(
        !editor(&state).reasons.is_empty(),
        "a new mode is not ready yet"
    );
}

#[test]
fn a_linear_mode_built_from_messages_is_saved_and_a_broken_one_is_not() {
    let (mut state, _) = send(State::default(), LearnMsg::Open);
    let steps = [
        LearnMsg::Name("Interleave".into()),
        LearnMsg::Kind(NodeKind::Start),
        LearnMsg::AddNode,
        LearnMsg::Kind(NodeKind::Present),
        LearnMsg::Label("show".into()),
        LearnMsg::AddNode,
        LearnMsg::Kind(NodeKind::Stop),
        LearnMsg::AddNode,
    ];
    for m in steps {
        state = send(state, m).0;
    }
    // Only the nodes exist so far: nothing is wired, so the mode cannot run.
    let (broken, fx) = send(state.clone(), LearnMsg::Save);
    assert!(fx.is_empty(), "a mode with reasons is not saved");
    assert!(
        editor(&broken)
            .notice
            .as_deref()
            .unwrap()
            .contains("Not saved")
    );

    for (from, to) in [("n1", "n2"), ("n2", "n3")] {
        state = send(state, LearnMsg::From(from.into())).0;
        state = send(state, LearnMsg::To(to.into())).0;
        state = send(state, LearnMsg::AddWire).0;
    }
    assert!(
        editor(&state).reasons.is_empty(),
        "{:?}",
        editor(&state).reasons
    );
    let (_, fx) = send(state, LearnMsg::Save);
    match &fx[..] {
        [Effect::Learn(LearnEffect::Save(g, _))] => {
            assert_eq!(g.name, "Interleave");
            assert_eq!(g.nodes.len(), 3);
            assert_eq!(g.edges.len(), 2);
        }
        other => panic!("expected a save, got {other:?}"),
    }
}

#[test]
fn removing_a_node_takes_its_wires_and_a_branch_needs_a_default() {
    let (mut state, _) = send(State::default(), LearnMsg::Open);
    for m in [
        LearnMsg::Kind(NodeKind::Start),
        LearnMsg::AddNode,
        LearnMsg::Kind(NodeKind::Branch),
        LearnMsg::AddNode,
        LearnMsg::Kind(NodeKind::Stop),
        LearnMsg::AddNode,
        LearnMsg::From("n1".into()),
        LearnMsg::To("n2".into()),
        LearnMsg::AddWire,
        LearnMsg::From("n2".into()),
        LearnMsg::To("n3".into()),
        LearnMsg::When("again".into()),
        LearnMsg::AddWire,
    ] {
        state = send(state, m).0;
    }
    assert!(
        editor(&state)
            .reasons
            .iter()
            .any(|r| r.contains("no default way on"))
    );
    state = send(state, LearnMsg::RemoveNode("n2".into())).0;
    let e = editor(&state);
    assert!(e.graph.edges.is_empty(), "the wires of n2 went with it");
    assert_eq!(e.graph.nodes.len(), 2);
}

#[test]
fn start_needs_a_saved_mode_with_no_reasons_and_then_closes_the_dialog() {
    let (state, _) = send(State::default(), LearnMsg::Open);
    let (held, fx) = send(state.clone(), LearnMsg::Start);
    assert!(fx.is_empty(), "an unsaved mode does not start");
    assert!(
        editor(&held)
            .notice
            .as_deref()
            .unwrap()
            .contains("Save the mode")
    );

    // A saved mode: the editor has a file name and no reasons.
    let mut saved = state;
    if let Some(Modal::LearnEditor(e)) = saved.modal.as_mut() {
        e.file = Some("quick-check.toml".into());
        e.reasons.clear();
    }
    let (after, fx) = send(saved, LearnMsg::Start);
    assert!(after.modal.is_none(), "the dialog closes");
    assert!(after.rehearse.starting);
    assert_eq!(
        fx,
        vec![Effect::Learn(LearnEffect::Start("quick-check.toml".into()))]
    );
}

#[test]
fn the_report_is_read_only_for_a_saved_mode_and_only_when_asked() {
    let (state, _) = send(State::default(), LearnMsg::Open);
    let (unsaved, fx) = send(state.clone(), LearnMsg::ShowReport);
    assert!(
        fx.is_empty(),
        "nothing is read for a mode that is not saved"
    );
    assert!(editor(&unsaved).report.is_none());

    let mut saved = state;
    if let Some(Modal::LearnEditor(e)) = saved.modal.as_mut() {
        e.file = Some("quick-check.toml".into());
    }
    let (_, fx) = send(saved, LearnMsg::ShowReport);
    assert_eq!(
        fx,
        vec![Effect::Learn(LearnEffect::Report("quick-check".into()))]
    );
}

#[test]
fn the_canvas_places_moves_selects_connects_and_deletes_steps() {
    let (mut state, _) = send(State::default(), LearnMsg::Open);
    // A click on empty canvas adds a step of the chosen kind at that place.
    state = send(state, LearnMsg::Kind(NodeKind::Start)).0;
    state = send(state, LearnMsg::PlaceAt(100.0, 50.0)).0;
    state = send(state, LearnMsg::Kind(NodeKind::Stop)).0;
    state = send(state, LearnMsg::PlaceAt(300.0, 50.0)).0;
    assert_eq!(editor(&state).graph.nodes.len(), 2);
    assert_eq!(editor(&state).layout.get("n1"), Some(&[100.0, 50.0]));

    // A drag moves a step by the drag's offset; a step with no saved place starts from its grid place.
    state = send(state, LearnMsg::MoveBy("n2".into(), 10.0, -5.0)).0;
    assert_eq!(editor(&state).layout.get("n2"), Some(&[310.0, 45.0]));

    // A click selects; Delete removes the selected step with its place.
    state = send(state, LearnMsg::ClickNode("n1".into())).0;
    assert_eq!(editor(&state).selected.as_deref(), Some("n1"));
    state = send(state, LearnMsg::DeleteSelected).0;
    assert_eq!(editor(&state).graph.nodes.len(), 1);
    assert!(!editor(&state).layout.contains_key("n1"));
    assert!(editor(&state).selected.is_none());
}

#[test]
fn in_connect_mode_two_clicks_draw_a_wire_with_the_chosen_condition() {
    let (mut state, _) = send(State::default(), LearnMsg::Open);
    state = send(state, LearnMsg::Kind(NodeKind::Start)).0;
    state = send(state, LearnMsg::AddNode).0;
    state = send(state, LearnMsg::Kind(NodeKind::Present)).0;
    state = send(state, LearnMsg::AddNode).0;
    state = send(state, LearnMsg::ToggleConnect).0;
    // Placing is off while connecting: a click on empty canvas adds nothing.
    state = send(state, LearnMsg::PlaceAt(5.0, 5.0)).0;
    assert_eq!(editor(&state).graph.nodes.len(), 2);

    state = send(state, LearnMsg::ClickNode("n1".into())).0;
    state = send(state, LearnMsg::ClickNode("n2".into())).0;
    let e = editor(&state);
    assert_eq!(e.graph.edges.len(), 1);
    assert_eq!(
        (e.graph.edges[0].from.as_str(), e.graph.edges[0].to.as_str()),
        ("n1", "n2")
    );
    assert!(
        e.from.is_empty() && e.to.is_empty(),
        "ready for the next wire"
    );
}
