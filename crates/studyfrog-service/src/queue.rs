//! Building a run from the library and the stored schedule.

use rusqlite::Connection;
use studyfrog_core::arena::{arena_get, arena_iter};
use studyfrog_core::library::Library;
use studyfrog_core::meta::Meta;
use studyfrog_core::plan::{ItemKind, RunPlan};
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budget, Candidate};
use studyfrog_srs::params::Params;
use studyfrog_store::error::StoreError;
use studyfrog_store::index::item_positions;
use studyfrog_store::schedule::load_all_mem_states;

use crate::queue_data::{Positions, Schedules};

fn candidate<'a>(
    meta: &'a Meta,
    kind: ItemKind,
    stack: Uid,
    pos: &Positions,
    sched: &Schedules,
) -> Candidate<'a> {
    let stored = sched.get(&meta.uid);
    Candidate {
        uid: meta.uid,
        kind,
        stack,
        tags: &meta.tags,
        position: pos.get(&meta.uid).copied().unwrap_or(u32::MAX),
        memory: stored.map(|m| m.state),
        due: stored.map(|m| m.due),
        last_review: stored.map(|m| m.last_review),
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

/// What to build: plan, presentation mode, identity, "now" and the daily budgets.
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
    /// Daily allowances (only the spaced order uses them).
    pub budget: Budget,
}

/// Build a not-yet-started run: reads positions and schedules from the database, applies the plan.
pub fn new_run(
    conn: &Connection,
    lib: &Library,
    req: NewRun,
    params: &Params,
) -> Result<Run, StoreError> {
    let (pos, sched) = (item_positions(conn)?, load_all_mem_states(conn)?);
    let queue = build_queue(
        &candidates(lib, &pos, &sched),
        &req.plan,
        req.now,
        req.budget,
        params,
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
