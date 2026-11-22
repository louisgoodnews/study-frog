#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A custom learn mode decides the queue (STORY-0198): an item is shown again in the run only when its
//! graph walk ends at a rate node; a built-in run keeps its own requeue rule.

mod common;

use common::{go, session};
use studyfrog_core::event::EventKind;
use studyfrog_core::plan::{plan_sequential, plan_spaced};
use studyfrog_core::review::Rating::{self, Again, Good};
use studyfrog_core::run::RunMode;
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind};
use studyfrog_rehearsal::learn_path::new_path;
use studyfrog_rehearsal::session::Session;

fn node(id: &str, kind: NodeKind) -> Node {
    Node {
        id: id.into(),
        kind,
        label: id.into(),
    }
}

fn edge(from: &str, to: &str, when: Option<&str>) -> Edge {
    Edge {
        from: from.into(),
        to: to.into(),
        when: when.map(str::to_owned),
    }
}

/// Again loops back to the item's presentation; any other rating ends the item's path.
fn looping() -> LearnGraph {
    LearnGraph {
        name: "loop".into(),
        nodes: vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("rate", NodeKind::Rate),
            node("pick", NodeKind::Branch),
            node("end", NodeKind::Stop),
        ],
        edges: vec![
            edge("start", "show", None),
            edge("show", "rate", None),
            edge("rate", "pick", None),
            edge("pick", "show", Some("again")),
            edge("pick", "end", Some("default")),
        ],
    }
}

/// Run the queue with these ratings (one per shown item) and return the items shown, in order.
fn shown(s: &mut Session, ratings: &[Rating]) -> Vec<u128> {
    let mut out_items = Vec::new();
    let mut t = 0;
    let mut out = go(s, t, Action::Start);
    let mut ratings = ratings.iter();
    loop {
        out_items.extend(
            out.events
                .iter()
                .filter(|e| e.kind == EventKind::ItemShown)
                .filter_map(|e| e.item.map(|u| u.0)),
        );
        if out.events.iter().any(|e| e.kind == EventKind::RunFinished) {
            break;
        }
        let Some(r) = ratings.next() else { break };
        t += 10;
        go(s, t, Action::Flip);
        out = go(s, t + 1, Action::Rate(*r));
    }
    out_items
}

#[test]
fn again_into_the_loop_brings_the_item_back_and_good_out_does_not() {
    let mut s = session(RunMode::Flip, plan_sequential(), &[1, 2]);
    s.learn = Some(new_path(looping()));
    // Item 1 is rated Again (the loop brings it back), then Good; item 2 is rated Good.
    let items = shown(&mut s, &[Again, Good, Good, Good]);
    assert_eq!(items, vec![1, 2, 1], "item 1 returns once, item 2 leaves");
}

#[test]
fn without_a_graph_the_built_in_rule_still_requeues_on_again() {
    let mut s = session(RunMode::Flip, plan_spaced(), &[1]);
    let items = shown(&mut s, &[Again, Good]);
    assert_eq!(
        items.iter().filter(|i| **i == 1).count(),
        2,
        "built-in Again requeues"
    );
}
