//! A contrastive drill: a practice run of exactly the items given (two items that get confused, a few leeches),
//! one after the other (a forgotten one comes back), without touching the schedule (STORY-0104).

use studyfrog_core::plan::{RunPlan, plan_sequential};
use studyfrog_core::run::{Run, RunMode};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;

use crate::create::IdSource;
use crate::day::DayWindow;
use crate::queue::{NewRun, new_run};
use crate::rehearse::begin_run;
use crate::session::{ServiceError, VaultSession, current_settings};

/// Create and store a practice run of `items` in the order given (items the vault does not have are left out).
pub fn start_drill(
    session: &VaultSession,
    items: &[Uid],
    mode: RunMode,
    (now, day): (UnixMs, DayWindow),
    ids: IdSource,
) -> Result<Run, ServiceError> {
    let req = NewRun {
        plan: RunPlan {
            requeue_again: true,
            ..plan_sequential()
        },
        mode,
        uid: compose_ulid(ids.now_ms, ids.random),
        now,
        day,
        global_budget: budget_unlimited(),
    };
    let mut run =
        new_run(&session.conn, req, &current_settings(session)).map_err(ServiceError::Store)?;
    let found = run.queue.clone();
    run.queue = items
        .iter()
        .copied()
        .filter(|u| found.contains(u))
        .collect();
    if run.queue.is_empty() {
        return Err(ServiceError::Invalid(
            "none of these items can be rehearsed".to_owned(),
        ));
    }
    begin_run(&session.conn, &run).map_err(ServiceError::Store)?;
    Ok(run)
}
