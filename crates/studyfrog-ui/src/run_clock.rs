//! The clock of a run on screen: the time left of a countdown or the time used of a count up, the warnings, and the end of a
//! countdown. Pure; the time comes from `State::now`.

use studyfrog_core::pacing::Pacing;
use studyfrog_core::run::RunState;
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::pacing::{active_ms, remaining_ms};

use crate::effect::Effect;
use crate::run_state::{RunView, engine_ms};
use crate::state::State;

/// A warning about the time left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warning {
    /// One minute or less.
    OneMinute,
    /// Ten seconds or less.
    TenSeconds,
}

/// What the clock shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clock {
    /// "Time left 04:12" or "Time used 03:20".
    pub text: String,
    /// The warning to give, for a countdown near its end.
    pub warning: Option<Warning>,
}

/// "mm:ss" for milliseconds (rounded up to the second for a countdown by the caller).
pub fn mmss(ms: u32) -> String {
    let s = ms / 1000;
    format!("{:02}:{:02}", s / 60, s % 60)
}

/// The words of a warning, for the screen and for screen readers.
pub fn warning_text(w: Warning) -> &'static str {
    match w {
        Warning::OneMinute => "One minute left",
        Warning::TenSeconds => "Ten seconds left",
    }
}

/// The clock of the run, or `None` when the run has no clock or is not running.
pub fn clock_of(state: &State, run: &RunView) -> Option<Clock> {
    let s = &run.session;
    if !matches!(s.run.state, RunState::Running | RunState::Paused) {
        return None;
    }
    let t = engine_ms(state.now, run.started_time, s.last_t_ms);
    let t = if s.run.state == RunState::Paused {
        s.paused_at_ms.unwrap_or(t)
    } else {
        t
    };
    match s.run.plan.pacing {
        Pacing::Free => None,
        Pacing::CountUp => Some(Clock {
            text: format!("Time used {}", mmss(active_ms(s, t))),
            warning: None,
        }),
        Pacing::Countdown(_) => {
            let left = remaining_ms(s, t)?;
            let warning = match left {
                0 => None,
                1..=10_000 => Some(Warning::TenSeconds),
                10_001..=60_000 => Some(Warning::OneMinute),
                _ => None,
            };
            Some(Clock {
                text: format!("Time left {}", mmss(left.div_ceil(1000) * 1000)),
                warning,
            })
        }
    }
}

/// End a countdown whose time is up: the engine finishes the run on a tick at or after the limit.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    let up = state.rehearse.run.as_ref().is_some_and(|run| {
        let s = &run.session;
        let t = engine_ms(state.now, run.started_time, s.last_t_ms);
        s.run.state == RunState::Running && remaining_ms(s, t) == Some(0)
    });
    if up {
        crate::run_update::act(state, Action::Tick)
    } else {
        (state, Vec::new())
    }
}
