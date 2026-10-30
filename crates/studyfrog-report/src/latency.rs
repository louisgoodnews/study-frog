//! How long items take to answer. Pure; reads the run event log.

use studyfrog_core::event::{Difficulty, EventKind, RunEvent};
use studyfrog_core::uid::Uid;

/// Time from showing an item to revealing or answering it, with the user's mark for the item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlipSample {
    /// The item.
    pub item: Uid,
    /// How hard the user marked it while it was on screen.
    pub mark: Option<Difficulty>,
    /// Milliseconds, paused time excluded.
    pub ms: u32,
}

/// Upper edges of the histogram bins in milliseconds; the last bin is open-ended.
pub const EDGES_MS: [u32; 8] = [1_000, 2_000, 3_000, 5_000, 8_000, 13_000, 21_000, 34_000];

/// Percentiles of a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyStats {
    /// Number of samples.
    pub n: usize,
    /// Median.
    pub p50: u32,
    /// 90th percentile.
    pub p90: u32,
    /// 95th percentile.
    pub p95: u32,
    /// Slowest.
    pub max: u32,
}

/// Nearest-rank percentile of an ascending sample.
pub fn percentile(sorted: &[u32], pct: u32) -> Option<u32> {
    let n = sorted.len();
    (n > 0).then(|| sorted[((n * pct as usize).div_ceil(100)).clamp(1, n) - 1])
}

/// Percentiles of `ms` (sorted in place).
pub fn latency_stats(ms: &mut [u32]) -> Option<LatencyStats> {
    ms.sort_unstable();
    Some(LatencyStats {
        n: ms.len(),
        p50: percentile(ms, 50)?,
        p90: percentile(ms, 90)?,
        p95: percentile(ms, 95)?,
        max: *ms.last()?,
    })
}

/// How many samples fall in each bin of [`EDGES_MS`] (one more bin than edges).
pub fn histogram(ms: &[u32]) -> Vec<u32> {
    let mut bins = vec![0; EDGES_MS.len() + 1];
    for &m in ms {
        bins[EDGES_MS
            .iter()
            .position(|&e| m < e)
            .unwrap_or(EDGES_MS.len())] += 1;
    }
    bins
}

/// Time-to-reveal samples from event logs (grouped by run, in order). The mark is the last one given
/// before the item was left.
pub fn flip_samples(events: &[RunEvent]) -> Vec<FlipSample> {
    let mut out: Vec<FlipSample> = Vec::new();
    let (mut run, mut paused_total, mut paused_at) = (None, 0_u32, None::<u32>);
    let (mut shown, mut mark, mut open) =
        (None::<(Uid, u32, u32)>, None::<Difficulty>, None::<usize>);
    for e in events {
        if run != Some(e.run) {
            (run, paused_total, paused_at, shown, mark, open) =
                (Some(e.run), 0, None, None, None, None);
        }
        match &e.kind {
            EventKind::RunPaused => paused_at = Some(e.t_ms),
            EventKind::RunResumed => {
                paused_total += paused_at.take().map_or(0, |at| e.t_ms.saturating_sub(at));
            }
            EventKind::ItemShown => {
                (shown, mark, open) = (e.item.map(|i| (i, e.t_ms, paused_total)), None, None);
            }
            EventKind::ItemMarked(d) => {
                mark = Some(*d);
                if let Some(i) = open {
                    out[i].mark = mark;
                }
            }
            EventKind::ItemFlipped | EventKind::AnswerSubmitted(_) => {
                if let Some((item, t, paused)) = shown.filter(|(i, ..)| Some(*i) == e.item) {
                    let ms = e
                        .t_ms
                        .saturating_sub(t)
                        .saturating_sub(paused_total - paused);
                    out.push(FlipSample { item, mark, ms });
                    open = Some(out.len() - 1);
                }
            }
            _ => {}
        }
    }
    out
}
