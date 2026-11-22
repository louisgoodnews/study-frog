//! Why a learn-mode graph cannot run (STORY-0198): the refusals and the sentences that explain them. Pure.

/// Why a graph cannot run. A graph with no reasons runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// There is no start node.
    NoStart,
    /// More than one start node (their ids).
    SeveralStarts(Vec<String>),
    /// Two nodes share this id.
    DuplicateNode(String),
    /// A wire names a node that is not in the graph.
    UnknownNode {
        /// The wire's start.
        from: String,
        /// The wire's end.
        to: String,
        /// The id that is not a node.
        node: String,
    },
    /// The node cannot be reached from the start.
    Unreachable(String),
    /// The node is not a stop, but has no way on.
    DeadEnd(String),
    /// A stop node has a way on.
    StopHasExit(String),
    /// A branch has fewer than two ways on.
    BranchNeedsTwoWays(String),
    /// A loop (these node ids) has no stop node and no branch that can leave it.
    LoopWithoutStop(Vec<String>),
    /// A wire carries a condition that is not one of the ratings or `default`.
    UnknownCondition {
        /// The wire's start.
        from: String,
        /// The condition it carries.
        when: String,
    },
    /// A branch has no `default` way on, so some rating would have no next step.
    BranchNeedsDefault(String),
}

/// The sentence the learner or researcher reads for a refusal.
pub fn reason(r: &Refusal) -> String {
    match r {
        Refusal::NoStart => "the graph has no start node".to_owned(),
        Refusal::SeveralStarts(ids) => {
            format!("the graph has more than one start node: {}", ids.join(", "))
        }
        Refusal::DuplicateNode(id) => format!("two nodes share the id {id}"),
        Refusal::UnknownNode { from, to, node } => {
            format!("the wire {from} -> {to} names {node}, which is not a node")
        }
        Refusal::Unreachable(id) => format!("{id} cannot be reached from the start"),
        Refusal::DeadEnd(id) => format!("{id} has no way on, and it is not a stop"),
        Refusal::StopHasExit(id) => format!("the stop {id} has a way on"),
        Refusal::BranchNeedsTwoWays(id) => format!("the branch {id} needs at least two ways on"),
        Refusal::UnknownCondition { from, when } => {
            format!(
                "the wire from {from} has the condition {when}, which is not again, hard, good, easy or default"
            )
        }
        Refusal::BranchNeedsDefault(id) => format!("the branch {id} has no default way on"),
        Refusal::LoopWithoutStop(ids) => format!(
            "the loop {} has no stop and no branch that can leave it",
            ids.join(" -> ")
        ),
    }
}
