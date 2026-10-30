//! The end-of-run summary, computed from the event log. Pure.

use studyfrog_core::event::{EventKind, Latency, RunEvent};
use studyfrog_core::review::Rating;

/// What happened in a run.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Summary {
    /// Ratings given: Again, Hard, Good, Easy.
    pub rated: [usize; 4],
    /// Items skipped.
    pub skipped: usize,
    /// Free-text answers submitted.
    pub answered: usize,
    /// Median time to reveal or answer, in milliseconds (paused time excluded).
    pub median_latency_ms: Option<u32>,
    /// Time of the last event, in milliseconds from the start.
    pub total_ms: u32,
}

fn rating_index(r: Rating) -> usize {
    usize::from(studyfrog_core::review::rating_value(r)) - 1
}

/// Median of the latencies of items that were flipped or answered (skips are not recall attempts).
fn median(latencies: &[Latency]) -> Option<u32> {
    let mut ms: Vec<u32> = latencies
        .iter()
        .filter(|l| l.kind != studyfrog_core::event::LatencyKind::Skip)
        .map(|l| l.ms)
        .collect();
    ms.sort_unstable();
    ms.get(ms.len().checked_sub(1)? / 2).copied()
}

/// Summarise a run.
pub fn summarize(events: &[RunEvent], latencies: &[Latency]) -> Summary {
    let mut s = Summary {
        median_latency_ms: median(latencies),
        total_ms: events.last().map_or(0, |e| e.t_ms),
        ..Summary::default()
    };
    for e in events {
        match &e.kind {
            EventKind::ItemRated(r) => s.rated[rating_index(*r)] += 1,
            EventKind::ItemSkipped => s.skipped += 1,
            EventKind::AnswerSubmitted(_) => s.answered += 1,
            _ => {}
        }
    }
    s
}
