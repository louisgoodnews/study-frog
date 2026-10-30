#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers shared by the rehearsal integration tests.

use studyfrog_core::event::event_name;
use studyfrog_core::plan::RunPlan;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::{Action, Input, StepOutput};
use studyfrog_rehearsal::session::{Session, session_new};
use studyfrog_rehearsal::step::step;

pub fn session(mode: RunMode, plan: RunPlan, queue: &[u128]) -> Session {
    let queue = queue.iter().map(|n| Uid(*n)).collect();
    session_new(Run {
        uid: Uid(100),
        mode,
        plan,
        queue,
        state: RunState::Idle,
        started_at: None,
    })
}

pub fn at(t_ms: u32, action: Action) -> Input {
    Input { t_ms, action }
}

pub fn go(s: &mut Session, t: u32, action: Action) -> StepOutput {
    step(s, at(t, action)).unwrap()
}

pub fn names(out: &StepOutput) -> Vec<&'static str> {
    out.events.iter().map(|e| event_name(&e.kind)).collect()
}
