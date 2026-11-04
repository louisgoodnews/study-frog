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
use studyfrog_store::schedule::Scheduled;
use studyfrog_store::schedule_stacks::load_stack_schedules;
use studyfrog_store::settings::{daily_counts, stack_limits};
use studyfrog_store::stack_scope::stack_homes;

use crate::day::DayWindow;
use crate::expiry::{expired_stacks, today_of, without_expired};
use crate::fam_all::reference_familiarities;
use crate::queue_data::StackSchedules;
use crate::settings::{params_for, scheduler_for};

/// The schedule a run sees for an item, from the selected stacks that hold it: the earliest due date among them,
/// and none (a new item) when any of them has never scheduled it. Also the stack that schedule belongs to.
fn merged(it: &IndexedItem, sched: &StackSchedules, selection: &[Uid]) -> (Uid, Option<Scheduled>) {
    let holders: Vec<Uid> = std::iter::once(it.stack)
        .chain(it.also_in.iter().copied())
        .filter(|s| selection.is_empty() || selection.contains(s))
        .collect();
    let found: Vec<(Uid, Option<&Scheduled>)> = holders
        .iter()
        .map(|s| (*s, sched.get(&(it.uid, *s))))
        .collect();
    if found.iter().any(|(_, s)| s.is_none()) {
        let new_in = found.iter().find(|(_, s)| s.is_none()).map(|(s, _)| *s);
        return (new_in.unwrap_or(it.stack), None);
    }
    found
        .into_iter()
        .filter_map(|(stack, s)| s.map(|s| (stack, *s)))
        .min_by_key(|(_, s)| s.due)
        .map_or((it.stack, None), |(stack, s)| (stack, Some(s)))
}

/// Queue candidates from the indexed items and their schedules in the stacks the run covers. An item held by
/// several of them is one candidate, due when it is due in any, counted against the stack that makes it due.
pub fn candidates<'a>(
    items: &'a [IndexedItem],
    (sched, selection): (&StackSchedules, &[Uid]),
    fams: &HashMap<(Uid, Uid), f32>,
) -> Vec<Candidate<'a>> {
    items
        .iter()
        .map(|it| {
            let (stack, scheduled) = merged(it, sched, selection);
            Candidate {
                uid: it.uid,
                kind: it.kind,
                stack,
                also_in: &it.also_in,
                tags: &it.tags,
                position: it.position,
                memory: scheduled.and_then(|s| s.memory),
                due: scheduled.map(|s| s.due),
                last_review: scheduled.map(|s| s.last_review),
                priority: it.priority,
                difficulty_hint: it.difficulty,
                familiarity: fams.get(&(it.uid, stack)).copied(),
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
    let homes = stack_homes(conn)?;
    let sched = load_stack_schedules(conn, |stack| scheduler_for(vault, homes.get(&stack)))?;
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
        reference_familiarities(conn, &items, req.now)?
    } else {
        HashMap::new()
    };
    let queue = build_queue(
        &candidates(&items, (&sched, &req.plan.filter.stacks), &fams),
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
