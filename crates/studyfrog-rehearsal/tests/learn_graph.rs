#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Learn-mode graphs (STORY-0198): a graph that cannot run is refused with the reason, and the shapes the
//! first version allows (linear, branching, a loop that a branch can leave) pass.

use studyfrog_rehearsal::learn_graph::{Edge, LearnGraph, Node, NodeKind, Refusal, check, reason};

fn node(id: &str, kind: NodeKind) -> Node {
    Node {
        id: id.into(),
        kind,
        label: id.into(),
    }
}

fn wire(from: &str, to: &str) -> Edge {
    Edge {
        from: from.into(),
        to: to.into(),
        when: None,
    }
}

fn when(from: &str, to: &str, cond: &str) -> Edge {
    Edge {
        from: from.into(),
        to: to.into(),
        when: Some(cond.into()),
    }
}

fn graph(nodes: Vec<Node>, edges: Vec<Edge>) -> LearnGraph {
    LearnGraph {
        name: "test".into(),
        nodes,
        edges,
    }
}

#[test]
fn a_linear_graph_runs() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("ask", NodeKind::Recall),
            node("rate", NodeKind::Rate),
            node("next", NodeKind::Schedule),
            node("end", NodeKind::Stop),
        ],
        vec![
            wire("start", "show"),
            wire("show", "ask"),
            wire("ask", "rate"),
            wire("rate", "next"),
            wire("next", "end"),
        ],
    );
    assert!(check(&g).is_empty(), "{:?}", check(&g));
}

#[test]
fn a_branch_with_two_ways_on_runs() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("rate", NodeKind::Rate),
            node("pick", NodeKind::Branch),
            node("again", NodeKind::Schedule),
            node("good", NodeKind::Schedule),
            node("end", NodeKind::Stop),
        ],
        vec![
            wire("start", "rate"),
            wire("rate", "pick"),
            when("pick", "again", "again"),
            when("pick", "good", "default"),
            wire("again", "end"),
            wire("good", "end"),
        ],
    );
    assert!(check(&g).is_empty(), "{:?}", check(&g));
}

#[test]
fn a_loop_that_a_branch_can_leave_runs() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("rate", NodeKind::Rate),
            node("pick", NodeKind::Branch),
            node("next", NodeKind::Schedule),
            node("end", NodeKind::Stop),
        ],
        vec![
            wire("start", "show"),
            wire("show", "rate"),
            wire("rate", "pick"),
            when("pick", "show", "again"),
            when("pick", "next", "default"),
            wire("next", "end"),
        ],
    );
    assert!(check(&g).is_empty(), "{:?}", check(&g));
}

#[test]
fn a_graph_without_a_start_is_refused_with_the_reason() {
    let g = graph(
        vec![node("show", NodeKind::Present), node("end", NodeKind::Stop)],
        vec![wire("show", "end")],
    );
    let refusals = check(&g);
    assert!(refusals.contains(&Refusal::NoStart));
    assert!(reason(&Refusal::NoStart).contains("no start"));
}

#[test]
fn two_starts_are_refused() {
    let g = graph(
        vec![
            node("a", NodeKind::Start),
            node("b", NodeKind::Start),
            node("end", NodeKind::Stop),
        ],
        vec![wire("a", "end"), wire("b", "end")],
    );
    assert!(
        check(&g)
            .iter()
            .any(|r| matches!(r, Refusal::SeveralStarts(ids) if ids.len() == 2))
    );
}

#[test]
fn a_loop_without_a_stop_is_refused_with_its_nodes() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("show", NodeKind::Present),
            node("ask", NodeKind::Recall),
        ],
        vec![
            wire("start", "show"),
            wire("show", "ask"),
            wire("ask", "show"),
        ],
    );
    let refusals = check(&g);
    let looped = refusals
        .iter()
        .find_map(|r| match r {
            Refusal::LoopWithoutStop(ids) => Some(ids.clone()),
            _ => None,
        })
        .expect("the loop is refused");
    assert_eq!(looped, vec!["ask".to_owned(), "show".to_owned()]);
    assert!(reason(&refusals[0]).len() > 10);
}

#[test]
fn a_wire_to_an_unknown_node_is_refused() {
    let g = graph(
        vec![node("start", NodeKind::Start), node("end", NodeKind::Stop)],
        vec![wire("start", "ghost"), wire("start", "end")],
    );
    assert!(check(&g).iter().any(|r| matches!(
        r,
        Refusal::UnknownNode { node, .. } if node == "ghost"
    )));
}

#[test]
fn a_node_with_no_way_on_is_a_dead_end_and_an_island_is_unreachable() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("stuck", NodeKind::Present),
            node("end", NodeKind::Stop),
            node("island", NodeKind::Present),
            node("island-end", NodeKind::Stop),
        ],
        vec![
            wire("start", "stuck"),
            wire("start", "end"),
            wire("island", "island-end"),
        ],
    );
    let refusals = check(&g);
    assert!(refusals.contains(&Refusal::DeadEnd("stuck".into())));
    assert!(refusals.contains(&Refusal::Unreachable("island".into())));
}

#[test]
fn a_stop_with_a_way_on_and_a_branch_with_one_way_are_refused() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("pick", NodeKind::Branch),
            node("end", NodeKind::Stop),
            node("after", NodeKind::Log),
        ],
        vec![
            wire("start", "pick"),
            when("pick", "end", "any"),
            wire("end", "after"),
        ],
    );
    let refusals = check(&g);
    assert!(refusals.contains(&Refusal::BranchNeedsTwoWays("pick".into())));
    assert!(refusals.contains(&Refusal::StopHasExit("end".into())));
}

#[test]
fn two_nodes_with_one_id_are_refused() {
    let g = graph(
        vec![node("a", NodeKind::Start), node("a", NodeKind::Stop)],
        vec![],
    );
    assert!(check(&g).contains(&Refusal::DuplicateNode("a".into())));
}

#[test]
fn an_unknown_condition_and_a_branch_without_default_are_refused() {
    let g = graph(
        vec![
            node("start", NodeKind::Start),
            node("pick", NodeKind::Branch),
            node("a", NodeKind::Stop),
            node("b", NodeKind::Stop),
        ],
        vec![
            wire("start", "pick"),
            when("pick", "a", "maybe"),
            when("pick", "b", "again"),
        ],
    );
    let refusals = check(&g);
    assert!(refusals.contains(&Refusal::UnknownCondition {
        from: "pick".into(),
        when: "maybe".into()
    }));
    assert!(refusals.contains(&Refusal::BranchNeedsDefault("pick".into())));
}
