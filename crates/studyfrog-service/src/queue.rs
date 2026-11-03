//! Building a run from the library and the stored schedule.

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::plan::{Order, RunPlan};
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::settings::VaultSettings;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budget, Budgets, Candidate};
use studyfrog_store::error::StoreError;
use studyfrog_store::overview::{IndexedItem, load_indexed_items};
use studyfrog_store::schedule::load_schedules;
use studyfrog_store::settings::{daily_counts, item_homes, stack_limits};

use crate::day::DayWindow;
use crate::expiry::{expired_stacks, today_of, without_expired};
use crate::fam_all::item_familiarities;
use crate::queue_data::Schedules;
use crate::settings::{params_for, scheduler_for};

/// Queue candidates from the indexed items and their schedules.
pub fn candidates<'a>(
    items: &'a [IndexedItem],
    sched: &Schedules,
    fams: &HashMap<Uid, f32>,
) -> Vec<Candidate<'a>> {
    items
        .iter()
        .map(|it| {
            let scheduled = sched.get(&it.uid);
            Candidate {
                uid: it.uid,
                kind: it.kind,
                stack: it.stack,
                also_in: &it.also_in,
                tags: &it.tags,
                position: it.position,
                memory: scheduled.and_then(|s| s.memory),
                due: scheduled.map(|s| s.due),
                last_review: scheduled.map(|s| s.last_review),
                priority: it.priority,
                difficulty_hint: it.difficulty,
                familiarity: fams.get(&it.uid).copied(),
            }
        })
        .collect()
}

/// What to build: plan, presentation mode, identity, "now", the day and an optional global cap.
#[derive(Debug, Clone, PartialEq)]
pub struct NewRun {
    /// How the queue is built and what the run does to the schedule.
    pub plan: RunPlan,
    /// Presentation mode.
    pub mode: RunMode,
    /// Identity of the run to create.
    pub uid: Uid,
    /// Wall-clock now, used for due checks.
    pub now: UnixMs,
    /// The day whose activity counts against the stacks' daily limits.
    pub day: DayWindow,
    /// Extra cap across all stacks (only the spaced order uses budgets).
    pub global_budget: Budget,
}

/// Remaining daily allowance of every stack that sets a limit: limit minus today's activity.
pub fn stack_budgets(
    conn: &Connection,
    day: DayWindow,
) -> Result<HashMap<Uid, Budget>, StoreError> {
    let counts = daily_counts(conn, day.start, day.end)?;
    let remaining = |limit: Option<u32>, done: u32| limit.map(|l| l.saturating_sub(done));
    Ok(stack_limits(conn)?
        .into_iter()
        .map(|(stack, lim)| {
            let done = counts.get(&stack).copied().unwrap_or_default();
            let budget = Budget {
                new_items: remaining(lim.new_items, done.new_done),
                reviews: remaining(lim.reviews, done.reviews_done),
            };
            (stack, budget)
        })
        .collect())
}

/// The spaced plan also leaves out what the vault's threshold calls familiar enough.
fn with_threshold(mut plan: RunPlan, vault: &VaultSettings) -> RunPlan {
    if plan.order == Order::Due && vault.familiarity_threshold < 1.0 {
        let bound = vault.familiarity_threshold;
        plan.filter.familiarity_below = Some(
            plan.filter
                .familiarity_below
                .map_or(bound, |b| b.min(bound)),
        );
    }
    plan
}

/// Whether building the queue reads familiarity (the spaced order, the least-familiar order, or a bound).
fn needs_familiarity(plan: &RunPlan) -> bool {
    matches!(plan.order, Order::Due | Order::LeastFamiliar)
        || plan.filter.familiarity_below.is_some()
}

/// Build a not-yet-started run: reads items, schedules and stack limits from the index and applies
/// the plan. No file is read.
pub fn new_run(conn: &Connection, req: NewRun, vault: &VaultSettings) -> Result<Run, StoreError> {
    let homes = item_homes(conn)?;
    let sched = load_schedules(conn, |uid| scheduler_for(vault, homes.get(&uid)))?;
    let budgets = Budgets {
        global: req.global_budget,
        per_stack: stack_budgets(conn, req.day)?,
    };
    let mut items = load_indexed_items(conn)?;
    if req.plan.filter.stacks.is_empty() {
        let expired = expired_stacks(conn, today_of(req.day))?;
        items = without_expired(items, &expired);
    }
    let plan = with_threshold(req.plan.clone(), vault);
    let fams = if needs_familiarity(&plan) {
        item_familiarities(conn, &items, req.now)?
    } else {
        HashMap::new()
    };
    let queue = build_queue(
        &candidates(&items, &sched, &fams),
        &plan,
        req.now,
        &budgets,
        &params_for(vault, None),
    );
    Ok(Run {
        uid: req.uid,
        mode: req.mode,
        plan: req.plan,
        queue,
        state: RunState::Idle,
        started_at: None,
    })
}

/// How many items a run with this request would contain.
pub fn preview_queue(
    conn: &Connection,
    req: NewRun,
    vault: &VaultSettings,
) -> Result<usize, StoreError> {
    new_run(conn, req, vault).map(|run| run.queue.len())
}
