//! Within-session fatigue: recall and answer time by how long a run has been going and by the
//! position of the item in it, the point where they get worse, and a live check for a slow answer
//! (STORY-0103). Pure.

use studyfrog_core::review::Rating;

use crate::latency::percentile;
use crate::session_log::SessionSample;

/// Lower edges of the elapsed-time buckets in minutes; the last one is open-ended.
pub const TIME_EDGES_MIN: [u32; 6] = [0, 5, 10, 15, 20, 30];
/// Lower edges of the position buckets; the last one is open-ended.
pub const POSITION_EDGES: [u32; 5] = [1, 11, 21, 36, 51];
/// Reviews a bucket needs before it can be the fatigue point.
pub const MIN_PACE_REVIEWS: u32 = 10;
/// A drop in recall of this many percentage points against the first ten minutes is fatigue.
pub const RECALL_DROP_POINTS: f32 = 8.0;
/// A rise in the median answer time of this share against the first ten minutes is fatigue.
pub const LATENCY_RISE: f32 = 0.35;

/// The numbers of one bucket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaceBucket {
    /// Lower edge (minutes or position).
    pub from: u32,
    /// Upper edge, exclusive; `u32::MAX` when open-ended.
    pub to: u32,
    /// Rated items.
    pub reviews: u32,
    /// Not rated Again.
    pub recalled: u32,
    /// Rated Good or Easy.
    pub good_easy: u32,
    /// Median time to reveal.
    pub median_ms: Option<u32>,
    /// 90th percentile of the time to reveal.
    pub p90_ms: Option<u32>,
}

/// Where fatigue sets in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inflection {
    /// Index into `by_time`.
    pub bucket: usize,
    /// Minutes into a run where it starts: the recommended session length.
    pub minutes: u32,
    /// Typical items rated before then (median over the runs that got there).
    pub items: Option<u32>,
    /// Recall drop against the first ten minutes, in percentage points.
    pub recall_drop_points: f32,
    /// Rise of the median answer time against the first ten minutes (0.42 is 42 %).
    pub latency_rise: f32,
}

/// The fatigue report.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FatigueReport {
    /// By minutes into the run.
    pub by_time: Vec<PaceBucket>,
    /// By position of the item in the run.
    pub by_position: Vec<PaceBucket>,
    /// The first ten minutes together.
    pub baseline: PaceBucket,
    /// Runs with at least one rated item.
    pub sessions: u32,
    /// Where it gets worse, if it does.
    pub inflection: Option<Inflection>,
}

/// Recall share of a bucket.
pub fn pace_rate(b: &PaceBucket) -> Option<f32> {
    (b.reviews > 0).then(|| b.recalled as f32 / b.reviews as f32)
}

fn bucket_of(edges: &[u32], x: u32) -> usize {
    edges.iter().rposition(|e| x >= *e).unwrap_or(0)
}

fn fill(
    edges: &[u32],
    samples: &[&SessionSample],
    key: impl Fn(&SessionSample) -> u32,
) -> Vec<PaceBucket> {
    let mut ms: Vec<Vec<u32>> = vec![Vec::new(); edges.len()];
    let mut out: Vec<PaceBucket> = (0..edges.len())
        .map(|i| PaceBucket {
            from: edges[i],
            to: edges.get(i + 1).copied().unwrap_or(u32::MAX),
            ..PaceBucket::default()
        })
        .collect();
    for s in samples {
        let i = bucket_of(edges, key(s));
        add(&mut out[i], s);
        ms[i].extend(s.latency_ms);
    }
    out.iter_mut()
        .zip(ms.iter_mut())
        .for_each(|(b, m)| percentiles(b, m));
    out
}

fn add(b: &mut PaceBucket, s: &SessionSample) {
    b.reviews += 1;
    b.recalled += u32::from(s.rating != Rating::Again);
    b.good_easy += u32::from(matches!(s.rating, Rating::Good | Rating::Easy));
}

fn percentiles(b: &mut PaceBucket, ms: &mut [u32]) {
    ms.sort_unstable();
    (b.median_ms, b.p90_ms) = (percentile(ms, 50), percentile(ms, 90));
}

fn minutes(s: &SessionSample) -> u32 {
    s.active_ms / 60_000
}

fn worse(b: &PaceBucket, base: &PaceBucket) -> Option<(f32, f32)> {
    let drop = (pace_rate(base)? - pace_rate(b)?) * 100.0;
    let rise = match (b.median_ms, base.median_ms) {
        (Some(m), Some(m0)) if m0 > 0 => m as f32 / m0 as f32 - 1.0,
        _ => 0.0,
    };
    (drop >= RECALL_DROP_POINTS || rise >= LATENCY_RISE).then_some((drop, rise))
}

fn items_before(samples: &[&SessionSample], cap_ms: u32) -> Option<u32> {
    let mut runs: Vec<(studyfrog_core::uid::Uid, u32, bool)> = Vec::new();
    for s in samples {
        if runs.last().is_none_or(|r| r.0 != s.run) {
            runs.push((s.run, 0, false));
        }
        if let Some(r) = runs.last_mut() {
            r.1 += u32::from(s.active_ms < cap_ms);
            r.2 |= s.active_ms >= cap_ms;
        }
    }
    let mut counts: Vec<u32> = runs.iter().filter(|r| r.2).map(|r| r.1).collect();
    counts.sort_unstable();
    percentile(&counts, 50)
}

/// The fatigue report of the samples of every run (grouped by run, in order).
pub fn fatigue_report(samples: &[SessionSample]) -> FatigueReport {
    let all: Vec<&SessionSample> = samples.iter().collect();
    let by_time = fill(&TIME_EDGES_MIN, &all, minutes);
    let by_position = fill(&POSITION_EDGES, &all, |s| s.position);
    let first: Vec<&SessionSample> = all.iter().copied().filter(|s| minutes(s) < 10).collect();
    let baseline = fill(&[0], &first, |_| 0).pop().unwrap_or_default();
    let inflection = by_time
        .iter()
        .enumerate()
        .skip(2)
        .filter(|(_, b)| b.reviews >= MIN_PACE_REVIEWS)
        .find_map(|(i, b)| worse(b, &baseline).map(|w| (i, b.from, w)))
        .map(|(bucket, minutes, (drop, rise))| Inflection {
            bucket,
            minutes,
            items: items_before(&all, minutes * 60_000),
            recall_drop_points: drop,
            latency_rise: rise,
        });
    let mut runs: Vec<_> = samples.iter().map(|s| s.run).collect();
    runs.dedup();
    FatigueReport {
        by_time,
        by_position,
        baseline,
        sessions: runs.len() as u32,
        inflection,
    }
}

/// How many times slower than usual `latest` is, when it is a spike: at least five earlier answers in
/// this run, at least twice their median and at least five seconds more. `None` otherwise.
pub fn latency_spike(earlier: &[u32], latest: u32) -> Option<f32> {
    if earlier.len() < 5 {
        return None;
    }
    let mut sorted = earlier.to_vec();
    sorted.sort_unstable();
    let median = percentile(&sorted, 50)?.max(1);
    (latest >= median * 2 && latest - median >= 5_000).then(|| latest as f32 / median as f32)
}
