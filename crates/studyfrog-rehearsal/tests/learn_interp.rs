#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The graph walk (STORY-0198): ratings pick the wire at a branch, a rate node waits for the learner, a stop
//! ends the walk, and a loop ends at its cap.

use std::collections::BTreeMap;

use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind};
use studyfrog_rehearsal::learn_interp::{End, walk};

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

/// start -> show -> rate -> pick; again loops to show (capped), default goes to next -> end.
fn looping() -> LearnGraph {
    LearnGraph {
        name: "loop".into(),
        nodes: vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("rate", NodeKind::Rate),
            node("pick", NodeKind::Branch),
            node("next", NodeKind::Schedule),
            node("end", NodeKind::Stop),
        ],
        edges: vec![
            edge("start", "show", None),
            edge("show", "rate", None),
            edge("rate", "pick", None),
            edge("pick", "show", Some("again")),
            edge("pick", "next", Some("default")),
            edge("next", "end", None),
        ],
    }
}

#[test]
fn the_walk_stops_at_the_first_rate_node() {
    let g = looping();
    let w = walk(&g, "start", "", &mut BTreeMap::new(), 3);
    assert_eq!(w.end, End::AtRate("rate".into()));
    assert_eq!(
        w.visits.iter().map(|v| v.node.as_str()).collect::<Vec<_>>(),
        ["show", "rate"]
    );
}

#[test]
fn a_rating_picks_the_branch_and_the_default_covers_the_rest() {
    let g = looping();
    let mut counts = BTreeMap::new();
    walk(&g, "start", "", &mut counts, 3);
    let good = walk(&g, "rate", "good", &mut counts, 3);
    assert_eq!(good.end, End::Stopped("end".into()));
    let next = good.visits.iter().find(|v| v.node == "next").unwrap();
    assert_eq!(next.via.as_deref(), Some("default"));

    let mut counts = BTreeMap::new();
    walk(&g, "start", "", &mut counts, 3);
    let again = walk(&g, "rate", "again", &mut counts, 3);
    assert_eq!(again.end, End::AtRate("rate".into()));
    let show = again.visits.iter().find(|v| v.node == "show").unwrap();
    assert_eq!(show.via.as_deref(), Some("again"));
}

#[test]
fn a_loop_ends_at_its_cap() {
    let g = looping();
    let mut counts = BTreeMap::new();
    walk(&g, "start", "", &mut counts, 2);
    let mut last = End::AtRate("rate".into());
    for _ in 0..5 {
        last = walk(&g, "rate", "again", &mut counts, 2).end;
        if matches!(last, End::CapReached(_)) {
            break;
        }
    }
    assert_eq!(last, End::CapReached("show".into()));
}
