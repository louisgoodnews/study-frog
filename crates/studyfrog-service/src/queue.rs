//! Building a run from the library and the stored schedule.

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::arena::{arena_get, arena_iter};
use studyfrog_core::library::Library;
use studyfrog_core::meta::Meta;
use studyfrog_core::plan::{ItemKind, RunPlan};
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::settings::VaultSettings;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budget, Budgets, Candidate};
use studyfrog_store::error::StoreError;
use studyfrog_store::index::item_positions;
use studyfrog_store::schedule::load_schedules;
use studyfrog_store::settings::{daily_counts, item_homes, stack_limits};

use crate::day::DayWindow;
use crate::queue_data::{Positions, Schedules};
use crate::settings::{params_for, scheduler_for};

fn candidate<'a>(
    meta: &'a Meta,
    kind: ItemKind,
    stack: Uid,
    pos: &Positions,
    sched: &Schedules,
) -> Candidate<'a> {
    let scheduled = sched.get(&meta.uid);
    Candidate {
        uid: meta.uid,
        kind,
        stack,
        tags: &meta.tags,
        position: pos.get(&meta.uid).copied().unwrap_or(u32::MAX),
        memory: scheduled.and_then(|s| s.memory),
        due: scheduled.map(|s| s.due),
        last_review: scheduled.map(|s| s.last_review),
    }
}

/// Every studiable item of the library as a queue candidate. Items missing from the index sort last.
pub fn candidates<'a>(lib: &'a Library, pos: &Positions, sched: &Schedules) -> Vec<Candidate<'a>> {
    let stack_of = |idx| arena_get(&lib.stacks, idx).map_or(Uid(0), |s| s.meta.uid);
    let notes = arena_iter(&lib.notes)
        .map(|(_, n)| candidate(&n.meta, ItemKind::Note, stack_of(n.stack), pos, sched));
    let cards = arena_iter(&lib.cards)
        .map(|(_, c)| candidate(&c.meta, ItemKind::Card, stack_of(c.stack), pos, sched));
    let questions = arena_iter(&lib.questions)
        .map(|(_, q)| candidate(&q.meta, ItemKind::Question, stack_of(q.stack), pos, sched));
    notes.chain(cards).chain(questions).collect()
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

/// Build a not-yet-started run: reads positions, schedules and stack limits from the database and
/// applies the plan.
pub fn new_run(
    conn: &Connection,
    lib: &Library,
    req: NewRun,
    vault: &VaultSettings,
) -> Result<Run, StoreError> {
    let homes = item_homes(conn)?;
    let sched = load_schedules(conn, |uid| scheduler_for(vault, homes.get(&uid)))?;
    let budgets = Budgets {
        global: req.global_budget,
        per_stack: stack_budgets(conn, req.day)?,
    };
    let queue = build_queue(
        &candidates(lib, &item_positions(conn)?, &sched),
        &req.plan,
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
