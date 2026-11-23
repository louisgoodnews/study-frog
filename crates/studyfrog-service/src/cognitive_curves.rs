//! The curves and distributions of the cognitive profile (BUG-0030): the learning velocity over
//! study time, the distribution of answer times and the semantic accuracy trend. Pure.

use studyfrog_core::review::Rating;
use studyfrog_store::reports::HistoryRow;
use studyfrog_store::telemetry::TelemetryPoint;

/// Milliseconds in a minute.
const MS_PER_MIN: f64 = 60_000.0;
/// Width of one answer-time bin, in milliseconds.
pub const BIN_MS: u32 = 2_000;
/// Number of answer-time bins; the last one holds every longer answer.
pub const BINS: usize = 12;

/// Whether a rating counts as recalled.
fn passed(r: Rating) -> bool {
    matches!(r, Rating::Good | Rating::Easy)
}

/// The learning velocity over study time: after each timed answer, the passing answers per minute of
/// answering so far. Points are (cumulative answering minutes, passing answers per minute), so the
/// last point is the profile's velocity figure and the curve shows whether it is rising.
pub fn velocity_curve(timed: &[(&HistoryRow, u32, u32)]) -> Vec<(f32, f32)> {
    let (mut minutes, mut passes) = (0.0_f64, 0.0_f64);
    timed
        .iter()
        .filter_map(|(r, answer_ms, _)| {
            minutes += f64::from(*answer_ms) / MS_PER_MIN;
            if passed(r.rating) {
                passes += 1.0;
            }
            (minutes > 0.0).then(|| (minutes as f32, (passes / minutes) as f32))
        })
        .collect()
}

/// How many timed answers fall in each answer-time bin of [`BIN_MS`] milliseconds.
pub fn latency_bins(timed: &[(&HistoryRow, u32, u32)]) -> Vec<u32> {
    let mut bins = vec![0_u32; BINS];
    for (_, answer_ms, _) in timed {
        let at = ((answer_ms / BIN_MS) as usize).min(BINS - 1);
        bins[at] += 1;
    }
    bins
}

/// The running mean of the semantic scores, in time order. Points are (number of scored answers so
/// far, mean score in `[0, 1]`). Answers without a score are left out.
pub fn semantic_trend(telemetry: &[TelemetryPoint]) -> Vec<(f32, f32)> {
    let (mut n, mut sum) = (0_u32, 0.0_f64);
    telemetry
        .iter()
        .filter_map(|p| p.semantic_score)
        .map(|s| {
            n += 1;
            sum += f64::from(s);
            (n as f32, (sum / f64::from(n)) as f32)
        })
        .collect()
}
