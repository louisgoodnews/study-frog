//! The graph's say over the queue (STORY-0198): for a custom run, a rating decides whether the item comes back
//! in this run. The walk is the same one the store records (`learn_interp::walk`), so the live queue and the stored
//! steps agree. Pure.

use std::collections::BTreeMap;

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;

use crate::learn_graph::{LearnGraph, NodeKind};
use crate::learn_interp::{End, walk};

/// How many times one item may enter one node in a run: the loop's stop rule.
pub const VISIT_CAP: u32 = 3;

/// Where one item stands in the graph: the rate node it waits at (`None` before its first walk, or once it has
/// left the run) and how often each node has been entered for it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ItemWalk {
    at: Option<String>,
    counts: BTreeMap<String, u32>,
}

/// A custom run's graph and each item's place in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnPath {
    /// The graph the run follows.
    pub graph: LearnGraph,
    items: BTreeMap<Uid, ItemWalk>,
}

/// A path over `graph`, with no item walked yet.
pub fn new_path(graph: LearnGraph) -> LearnPath {
    LearnPath {
        graph,
        items: BTreeMap::new(),
    }
}

/// The condition word a rating stands for on a branch wire.
pub fn rating_word(r: Rating) -> &'static str {
    match r {
        Rating::Again => "again",
        Rating::Hard => "hard",
        Rating::Good => "good",
        Rating::Easy => "easy",
    }
}

/// Walk `item` through the graph for its rating. Returns true when the walk ends at a rate node, so the item is
/// shown again in this run; false when the graph sends it out (a stop, a loop at its cap, or no way on).
pub fn rate(path: &mut LearnPath, item: Uid, rating: Rating) -> bool {
    let Some(start) = path
        .graph
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Start)
        .map(|n| n.id.clone())
    else {
        return false;
    };
    let graph = &path.graph;
    let state = path.items.entry(item).or_default();
    if state.at.is_none() {
        // The item's first rating: walk from the start to the rate node it waits at.
        match walk(graph, &start, "", &mut state.counts, VISIT_CAP).end {
            End::AtRate(node) => state.at = Some(node),
            _ => return false,
        }
    }
    let at = state.at.clone().unwrap_or_default();
    match walk(
        graph,
        &at,
        rating_word(rating),
        &mut state.counts,
        VISIT_CAP,
    )
    .end
    {
        End::AtRate(node) => {
            state.at = Some(node);
            true
        }
        _ => {
            state.at = None;
            false
        }
    }
}
