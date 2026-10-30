//! Queue construction: filter, order, cap. A pure function of its arguments.

use studyfrog_core::plan::{Order, RunPlan};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_srs::params::Params;

use crate::queue::candidate::{Budgets, Candidate};
use crate::queue::filter::admits;
use crate::queue::order::sort_by_order;
use crate::queue::shuffle::shuffle;
use crate::queue::spaced::spaced_queue;

/// Build the queue for `plan`. Every order except `Due` goes through the whole filtered list;
/// `Due` also applies the daily `budgets`. The result is capped at `plan.limit` and shuffled when
/// the plan asks for it.
pub fn build_queue(
    candidates: &[Candidate],
    plan: &RunPlan,
    now: UnixMs,
    budgets: &Budgets,
    params: &Params,
) -> Vec<Uid> {
    let admitted: Vec<&Candidate> = candidates
        .iter()
        .filter(|c| admits(&plan.filter, c, now))
        .collect();
    let sample = plan.shuffle && plan.order == Order::Sequential;
    let mut ordered = match plan.order {
        Order::Due => spaced_queue(&admitted, now, budgets),
        other => {
            let mut items = admitted;
            sort_by_order(&mut items, other, now, params);
            items
        }
    };
    // Sequential has no priority, so shuffling it first makes a limit pick a random sample; every
    // other order keeps its priority when the limit cuts, and the cut items are shuffled afterwards.
    if sample {
        shuffle(&mut ordered, plan.seed);
    }
    if let Some(limit) = plan.limit {
        ordered.truncate(limit as usize);
    }
    if plan.shuffle && !sample {
        shuffle(&mut ordered, plan.seed);
    }
    ordered.iter().map(|c| c.uid).collect()
}
