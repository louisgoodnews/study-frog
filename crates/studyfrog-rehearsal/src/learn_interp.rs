//! Walking a learn-mode graph (STORY-0198): from where an item stands, follow the wires one rating at a time
//! until the graph waits for the learner (a rate node) or ends (a stop, or a loop that reached its cap).
//! Pure: the caller keeps the visit counts and records what the walk returns.

use std::collections::BTreeMap;

use crate::learn_graph::{LearnGraph, NodeKind};

/// One node the walk entered, and the condition of the wire that led there (`None` for a plain wire).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visit {
    /// The node entered.
    pub node: String,
    /// The branch condition taken to get there.
    pub via: Option<String>,
}

/// Why a walk stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum End {
    /// Waiting for the learner's rating at this rate node.
    AtRate(String),
    /// Reached a stop node.
    Stopped(String),
    /// A node would be entered for the cap-th time; the walk ends there (a loop's stop rule).
    CapReached(String),
    /// The graph has no way on from this node (a graph the check would have refused).
    Stuck(String),
}

/// The result of a walk: the nodes entered, in order, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    /// The nodes entered, in order.
    pub visits: Vec<Visit>,
    /// How the walk ended.
    pub end: End,
}

/// Walk from `from` (the start node, or the rate node the item is waiting at) with a rating word
/// (`again`, `hard`, `good`, `easy`). `counts` holds how many times each node has been entered for this
/// item, and is updated; a node that would exceed `cap` ends the walk.
pub fn walk(
    g: &LearnGraph,
    from: &str,
    rating: &str,
    counts: &mut BTreeMap<String, u32>,
    cap: u32,
) -> Walk {
    let kind: BTreeMap<&str, NodeKind> = g.nodes.iter().map(|n| (n.id.as_str(), n.kind)).collect();
    let mut cur = from.to_owned();
    let mut via: Option<String> = None;
    let mut visits = Vec::new();
    // A walk never needs more steps than there are nodes times the cap, so this bound only guards a bad graph.
    for _ in 0..(g.nodes.len() as u32 * cap.max(1) + 1) {
        let kind_now = kind.get(cur.as_str()).copied();
        if kind_now == Some(NodeKind::Stop) {
            return Walk {
                visits,
                end: End::Stopped(cur),
            };
        }
        if kind_now == Some(NodeKind::Rate) && !visits.is_empty() {
            return Walk {
                visits,
                end: End::AtRate(cur),
            };
        }
        let Some(next) = pick(g, &cur, kind_now, rating) else {
            return Walk {
                visits,
                end: End::Stuck(cur),
            };
        };
        let entered = counts.entry(next.0.clone()).or_insert(0);
        if *entered >= cap {
            return Walk {
                visits,
                end: End::CapReached(next.0),
            };
        }
        *entered += 1;
        visits.push(Visit {
            node: next.0.clone(),
            via: next.1.clone(),
        });
        cur = next.0;
        via = next.1;
    }
    let _ = via;
    Walk {
        visits,
        end: End::Stuck(cur),
    }
}

/// The wire to follow from `cur`: for a branch, the one whose condition is the rating (else `default`);
/// otherwise the single wire out. Returns the next node and the condition taken.
fn pick(
    g: &LearnGraph,
    cur: &str,
    kind: Option<NodeKind>,
    rating: &str,
) -> Option<(String, Option<String>)> {
    let out: Vec<_> = g.edges.iter().filter(|e| e.from == cur).collect();
    if kind == Some(NodeKind::Branch) {
        let chosen = out
            .iter()
            .find(|e| e.when.as_deref() == Some(rating))
            .or_else(|| out.iter().find(|e| e.when.as_deref() == Some("default")))?;
        return Some((chosen.to.clone(), chosen.when.clone()));
    }
    out.first().map(|e| (e.to.clone(), None))
}
