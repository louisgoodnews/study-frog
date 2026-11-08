//! Every rated item of every run, placed in its run: its position, how long the run had been going
//! (paused time excluded), how long it took to reveal and to rate. Pure; reads the run event log.

use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;

/// One rated item in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSample {
    /// The run.
    pub run: Uid,
    /// The item.
    pub item: Uid,
    /// 1 for the first item shown in the run, and so on.
    pub position: u32,
    /// The run's clock when the item was shown (paused time included), to place it in wall time.
    pub shown_t_ms: u32,
    /// Active time of the run when the item was shown (paused time excluded).
    pub active_ms: u32,
    /// Time from showing to revealing or answering (paused time excluded), if it was revealed.
    pub latency_ms: Option<u32>,
    /// Time from showing to rating (paused time excluded).
    pub item_ms: u32,
    /// The rating.
    pub rating: Rating,
}

/// What is known about the item on screen.
#[derive(Clone, Copy)]
struct Open {
    item: Uid,
    t: u32,
    paused: u32,
    latency: Option<u32>,
}

#[derive(Default)]
struct RunClock {
    run: Option<Uid>,
    paused_total: u32,
    paused_at: Option<u32>,
    position: u32,
    open: Option<Open>,
}

fn active(clock: &RunClock, t: u32, since: &Open) -> u32 {
    t.saturating_sub(since.t)
        .saturating_sub(clock.paused_total - since.paused)
}

fn on_event(c: &mut RunClock, e: &RunEvent, out: &mut Vec<SessionSample>) {
    match &e.kind {
        EventKind::RunPaused => c.paused_at = Some(e.t_ms),
        EventKind::RunResumed => {
            c.paused_total += c.paused_at.take().map_or(0, |at| e.t_ms.saturating_sub(at));
        }
        EventKind::ItemShown => {
            c.position += 1;
            c.open = e.item.map(|item| Open {
                item,
                t: e.t_ms,
                paused: c.paused_total,
                latency: None,
            });
        }
        EventKind::ItemFlipped | EventKind::AnswerSubmitted(_) => {
            let lat = c.open.map(|o| active(c, e.t_ms, &o));
            if let Some(o) = c.open.as_mut().filter(|o| Some(o.item) == e.item) {
                o.latency = o.latency.or(lat);
            }
        }
        EventKind::ItemRated(rating) => rated(c, e, *rating, out),
        _ => {}
    }
}

fn rated(c: &mut RunClock, e: &RunEvent, rating: Rating, out: &mut Vec<SessionSample>) {
    let Some(o) = c.open.filter(|o| Some(o.item) == e.item) else {
        return;
    };
    out.push(SessionSample {
        run: e.run,
        item: o.item,
        position: c.position,
        shown_t_ms: o.t,
        active_ms: o.t.saturating_sub(o.paused),
        latency_ms: o.latency,
        item_ms: active(c, e.t_ms, &o),
        rating,
    });
}

/// The rated items of event logs grouped by run, in order.
pub fn session_samples(events: &[RunEvent]) -> Vec<SessionSample> {
    let (mut clock, mut out) = (RunClock::default(), Vec::new());
    for e in events {
        if clock.run != Some(e.run) {
            clock = RunClock {
                run: Some(e.run),
                ..RunClock::default()
            };
        }
        on_event(&mut clock, e, &mut out);
    }
    out
}
