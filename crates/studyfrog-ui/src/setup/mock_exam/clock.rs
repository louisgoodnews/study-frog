//! The exam clock (STORY-0185, BUG-0045): the time left counts down from the exam's total, and the exam is
//! submitted when the time runs out. The warnings at ten and two minutes are shown by the view from the same
//! reading. Pure: the app's time is passed in through the state.

use studyfrog_rehearsal::mock_timer::{TimerAlert, alerts_between};

use super::{MockExamUi, finish, with_ui};
use crate::effect::Effect;
use crate::state::State;

/// Whole seconds from `started` to `now` on the app clock, never negative.
pub fn elapsed_sec(now: f64, started: f64) -> u32 {
    let secs = (now - started).max(0.0).floor();
    secs.min(f64::from(u32::MAX)) as u32
}

/// Advance the clock of a running exam. When its time runs out, the exam is submitted, as "Finish and grade" is.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    let Some(ui) = state.rehearse.mock.clone() else {
        return (state, Vec::new());
    };
    let (Some(started), Some(g), true) =
        (ui.started_at, ui.result.as_ref(), ui.started && !ui.locked)
    else {
        return (state, Vec::new());
    };
    let limit = g.exam.time_limit_min * 60;
    let elapsed = elapsed_sec(state.now, started);
    let expired = alerts_between(limit, ui.elapsed_sec, elapsed).contains(&TimerAlert::Expired);
    let state = with_ui(
        state,
        Some(MockExamUi {
            elapsed_sec: elapsed,
            ..ui
        }),
    );
    if expired {
        finish::finish(state)
    } else {
        (state, Vec::new())
    }
}
