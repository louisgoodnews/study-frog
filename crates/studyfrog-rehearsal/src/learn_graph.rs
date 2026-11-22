//! Learn modes drawn as graphs (STORY-0198): the nodes and wires of a custom learn mode, and the rules that
//! refuse a graph that cannot run. Pure: no files and no screen. The first version allows linear and
//! branching graphs; a loop needs a stop rule (a stop node, or a branch that can leave the loop).

use std::collections::{BTreeMap, BTreeSet};

pub use crate::learn_refusal::{Refusal, reason};

/// What a node does in a run: one pure step the engine can test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// Where a run begins. Exactly one per graph.
    Start,
    /// Show the item's prompt.
    Present,
    /// Ask for the answer (flip, or typed).
    Recall,
    /// Take a rating from the learner.
    Rate,
    /// Schedule the next review from the rating.
    Schedule,
    /// Record a line in the run's log, kept apart from the built-in modes.
    Log,
    /// Choose the next step by a condition on the wire (`when`). At least two ways on.
    Branch,
    /// Where a run ends. No way on.
    Stop,
}

/// A node of a learn mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Unique within the graph.
    pub id: String,
    /// What it does.
    pub kind: NodeKind,
    /// The text shown on the canvas.
    pub label: String,
}

/// A wire from one node to another. `when` is the condition of a branch's wire (empty elsewhere).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// The node the wire leaves.
    pub from: String,
    /// The node the wire enters.
    pub to: String,
    /// The branch condition, such as `again` or `good`.
    pub when: Option<String>,
}

/// A custom learn mode.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LearnGraph {
    /// The name the mode is known by (and its file's name).
    pub name: String,
    /// The nodes.
    pub nodes: Vec<Node>,
    /// The wires.
    pub edges: Vec<Edge>,
}

/// The conditions a branch wire may carry: the four ratings, and `default` for any other.
pub const CONDITIONS: [&str; 5] = ["again", "hard", "good", "easy", "default"];

/// The condition rules: every wire's condition is a known word, and every branch has a default way on.
fn conditions(g: &LearnGraph) -> Vec<Refusal> {
    let mut out = Vec::new();
    for e in &g.edges {
        if let Some(w) = e.when.as_deref().filter(|w| !CONDITIONS.contains(w)) {
            out.push(Refusal::UnknownCondition {
                from: e.from.clone(),
                when: w.to_owned(),
            });
        }
    }
    for n in g.nodes.iter().filter(|n| n.kind == NodeKind::Branch) {
        let has_default = g
            .edges
            .iter()
            .any(|e| e.from == n.id && e.when.as_deref() == Some("default"));
        if !has_default {
            out.push(Refusal::BranchNeedsDefault(n.id.clone()));
        }
    }
    out
}

/// Every reason the graph cannot run, in a stable order; empty when it can.
pub fn check(g: &LearnGraph) -> Vec<Refusal> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for n in &g.nodes {
        if !seen.insert(n.id.as_str()) {
            out.push(Refusal::DuplicateNode(n.id.clone()));
        }
    }
    let starts: Vec<String> = g
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Start)
        .map(|n| n.id.clone())
        .collect();
    match starts.len() {
        0 => out.push(Refusal::NoStart),
        1 => {}
        _ => out.push(Refusal::SeveralStarts(starts.clone())),
    }
    for e in &g.edges {
        for node in [&e.from, &e.to] {
            if !seen.contains(node.as_str()) {
                out.push(Refusal::UnknownNode {
                    from: e.from.clone(),
                    to: e.to.clone(),
                    node: node.clone(),
                });
            }
        }
    }
    out.extend(conditions(g));
    let adj = adjacency(g);
    for n in &g.nodes {
        let ways = adj.get(n.id.as_str()).map_or(0, Vec::len);
        match n.kind {
            NodeKind::Stop if ways > 0 => out.push(Refusal::StopHasExit(n.id.clone())),
            NodeKind::Stop => {}
            NodeKind::Branch if ways < 2 => out.push(Refusal::BranchNeedsTwoWays(n.id.clone())),
            NodeKind::Branch => {}
            _ if ways == 0 => out.push(Refusal::DeadEnd(n.id.clone())),
            _ => {}
        }
    }
    if let Some(start) = starts.first().filter(|_| starts.len() == 1) {
        let reached = reach(&adj, start);
        for n in &g.nodes {
            if !reached.contains(n.id.as_str()) {
                out.push(Refusal::Unreachable(n.id.clone()));
            }
        }
    }
    out.extend(loops_without_stop(g, &adj));
    out
}

/// The wires that leave each node, by the ids they enter. Wires to unknown nodes are left out.
fn adjacency(g: &LearnGraph) -> BTreeMap<&str, Vec<&str>> {
    let ids: BTreeSet<&str> = g.nodes.iter().map(|n| n.id.as_str()).collect();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in &g.edges {
        if ids.contains(e.from.as_str()) && ids.contains(e.to.as_str()) {
            adj.entry(e.from.as_str()).or_default().push(e.to.as_str());
        }
    }
    adj
}

/// Every node reachable from `start`, the start included.
fn reach<'a>(adj: &BTreeMap<&'a str, Vec<&'a str>>, start: &'a str) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::from([start]);
    let mut stack = vec![start];
    while let Some(v) = stack.pop() {
        for &w in adj.get(v).map_or(&[][..], Vec::as_slice) {
            if seen.insert(w) {
                stack.push(w);
            }
        }
    }
    seen
}

/// One refusal per cycle group that has no stop: no stop node in it, and no branch in it that can leave it.
fn loops_without_stop(g: &LearnGraph, adj: &BTreeMap<&str, Vec<&str>>) -> Vec<Refusal> {
    let kinds: BTreeMap<&str, NodeKind> = g.nodes.iter().map(|n| (n.id.as_str(), n.kind)).collect();
    let mut groups: BTreeSet<BTreeSet<&str>> = BTreeSet::new();
    for n in &g.nodes {
        let id = n.id.as_str();
        let on_cycle = adj
            .get(id)
            .is_some_and(|succ| succ.iter().any(|s| reach(adj, s).contains(id)));
        if on_cycle {
            let group = reach(adj, id)
                .into_iter()
                .filter(|v| reach(adj, v).contains(id))
                .collect();
            groups.insert(group);
        }
    }
    let mut out = Vec::new();
    for group in groups {
        let stops = group.iter().any(|v| kinds.get(v) == Some(&NodeKind::Stop));
        let leaves = group.iter().any(|v| {
            kinds.get(v) == Some(&NodeKind::Branch)
                && adj
                    .get(v)
                    .is_some_and(|w| w.iter().any(|t| !group.contains(t)))
        });
        if !stops && !leaves {
            out.push(Refusal::LoopWithoutStop(
                group.iter().map(|s| s.to_string()).collect(),
            ));
        }
    }
    out
}
