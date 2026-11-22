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

/// Where a rated item goes in the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Back {
    /// Out of the run.
    Out,
    /// To the end of the queue (the built-in rule for Again).
    End,
    /// Back after this many other items (a custom mode's order).
    After(usize),
}

/// How many other items a custom run shows before an item comes back, by its rating: a poor rating returns sooner.
/// These are the defaults of this first version; the mode's graph decides whether an item comes back at all.
pub fn gap(rating: Rating) -> usize {
    match rating {
        Rating::Again => 2,
        Rating::Hard => 4,
        Rating::Good => 6,
        Rating::Easy => 8,
    }
}

/// Walk `item` through the graph for its rating. The item comes back after `gap(rating)` other items when the walk
/// ends at a rate node; it leaves the run when the graph sends it out (a stop, a loop at its cap, or no way on).
pub fn rate(path: &mut LearnPath, item: Uid, rating: Rating) -> Back {
    let Some(start) = path
        .graph
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Start)
        .map(|n| n.id.clone())
    else {
        return Back::Out;
    };
    let graph = &path.graph;
    let state = path.items.entry(item).or_default();
    if state.at.is_none() {
        // The item's first rating: walk from the start to the rate node it waits at.
        match walk(graph, &start, "", &mut state.counts, VISIT_CAP).end {
            End::AtRate(node) => state.at = Some(node),
            _ => return Back::Out,
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
            Back::After(gap(rating))
        }
        _ => {
            state.at = None;
            Back::Out
        }
    }
}
