//! Queue construction: filter, order, cap. A pure function of its arguments.

use studyfrog_core::plan::{Order, RunPlan};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_srs::params::Params;

use crate::queue::candidate::{Budget, Candidate};
use crate::queue::filter::admits;
use crate::queue::order::sort_by_order;
use crate::queue::spaced::spaced_queue;

/// Build the queue for `plan`. Every order except `Due` goes through the whole filtered list;
/// `Due` also applies the daily `budget`. The result is capped at `plan.limit`.
pub fn build_queue(
    candidates: &[Candidate],
    plan: &RunPlan,
    now: UnixMs,
    budget: Budget,
    params: &Params,
) -> Vec<Uid> {
    let admitted: Vec<&Candidate> = candidates
        .iter()
        .filter(|c| admits(&plan.filter, c, now))
        .collect();
    let mut ordered = match plan.order {
        Order::Due => spaced_queue(&admitted, now, budget),
        other => {
            let mut items = admitted;
            sort_by_order(&mut items, other, plan.seed, now, params);
            items
        }
    };
    if let Some(limit) = plan.limit {
        ordered.truncate(limit as usize);
    }
    ordered.iter().map(|c| c.uid).collect()
}
