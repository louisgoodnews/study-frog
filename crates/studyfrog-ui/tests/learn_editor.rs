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
        [Effect::Learn(LearnEffect::Save(g))] => {
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
