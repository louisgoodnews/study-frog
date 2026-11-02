//! The clock of a run: active time, the time a countdown has left, and the time a finished run used. Pure.

use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::pacing::limit_ms;

use crate::session::Session;

/// Active time at `t_ms`: the time since the run started minus all time paused. While paused the clock stands still.
pub fn active_ms(s: &Session, t_ms: u32) -> u32 {
    let ongoing_pause = s.paused_at_ms.map_or(0, |at| t_ms.saturating_sub(at));
    t_ms.saturating_sub(s.started_ms)
        .saturating_sub(s.paused_total_ms)
        .saturating_sub(ongoing_pause)
}

/// What a countdown has left at `t_ms`; `None` when the run is not a countdown.
pub fn remaining_ms(s: &Session, t_ms: u32) -> Option<u32> {
    limit_ms(s.run.plan.pacing).map(|limit| limit.saturating_sub(active_ms(s, t_ms)))
}

/// The active time a run used, from its events: from the start to the last event, without the time paused.
pub fn run_active_ms(events: &[RunEvent]) -> u32 {
    let Some(start) = events.iter().find(|e| e.kind == EventKind::RunStarted) else {
        return 0;
    };
    let end = events.iter().map(|e| e.t_ms).max().unwrap_or(start.t_ms);
    let mut paused = 0_u32;
    let mut since: Option<u32> = None;
    for e in events {
        match (&e.kind, since) {
            (EventKind::RunPaused, None) => since = Some(e.t_ms),
            (EventKind::RunResumed, Some(at)) => {
                paused = paused.saturating_add(e.t_ms.saturating_sub(at));
                since = None;
            }
            _ => {}
        }
    }
    if let Some(at) = since {
        paused = paused.saturating_add(end.saturating_sub(at));
    }
    end.saturating_sub(start.t_ms).saturating_sub(paused)
}
