//! The cognitive profile (STORY-0178): how the user recalls, read from the review history. Pure.
//!
//! The history holds the answer time, the time the words should take, and the rating. Semantic
//! scores and stability are not stored, so the profile uses the rating (passing means Good or Easy)
//! and the timing instead.

use studyfrog_core::review::Rating;
use studyfrog_report::difficulty::learning_velocity;
use studyfrog_store::reports::HistoryRow;

/// The most recent answers kept for the latency scatter.
const MAX_POINTS: usize = 500;
/// Milliseconds in a minute.
const MS_PER_MIN: f64 = 60_000.0;
/// Milliseconds in a week.
const WEEK_MS: i64 = 7 * 24 * 3_600_000;

/// Subjective ratings against objective timing, on the timed answers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calibration {
    /// Share of answers rated Easy.
    pub easy_share: f32,
    /// Share of answers given within the time the words should take.
    pub in_time_share: f32,
}

/// What the review history says about recall.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CognitiveProfile {
    /// Reviews in the history.
    pub answers: usize,
    /// Reviews with both an answer time and an expected time.
    pub timed: usize,
    /// Per timed answer (expected seconds, answer seconds), most recent [`MAX_POINTS`].
    pub length_points: Vec<(f32, f32)>,
    /// Passing answers gained per minute of answering time; `None` with too little data.
    pub velocity: Option<f64>,
    /// Per week since the first review: (week number, share of all answers so far that passed).
    pub weekly_pass: Vec<(f32, f32)>,
    /// Self-assessment against timing; `None` without timed answers.
    pub calibration: Option<Calibration>,
}

/// Whether a rating counts as recalled.
fn passed(r: Rating) -> bool {
    matches!(r, Rating::Good | Rating::Easy)
}

/// Build the profile from the review history, oldest first.
pub fn profile(rows: &[HistoryRow]) -> CognitiveProfile {
    let timed: Vec<(&HistoryRow, u32, u32)> = rows
        .iter()
        .filter_map(|r| Some((r, r.answer_ms?, r.expected_ms?)))
        .collect();
    let length_points: Vec<(f32, f32)> = timed
        .iter()
        .map(|(_, a, e)| (*e as f32 / 1000.0, *a as f32 / 1000.0))
        .collect();
    let keep = length_points.len().saturating_sub(MAX_POINTS);
    CognitiveProfile {
        answers: rows.len(),
        timed: timed.len(),
        length_points: length_points[keep..].to_vec(),
        velocity: velocity(&timed),
        weekly_pass: weekly_pass(rows),
        calibration: calibration(&timed),
    }
}

/// Slope of passing answers against cumulative answering minutes.
fn velocity(timed: &[(&HistoryRow, u32, u32)]) -> Option<f64> {
    let mut minutes = 0.0;
    let mut passes = 0.0;
    let points: Vec<(f64, f64)> = timed
        .iter()
        .map(|(r, a, _)| {
            minutes += f64::from(*a) / MS_PER_MIN;
            if passed(r.rating) {
                passes += 1.0;
            }
            (minutes, passes)
        })
        .collect();
    learning_velocity(&points)
}

/// The cumulative share of passing answers at the end of each week since the first review.
fn weekly_pass(rows: &[HistoryRow]) -> Vec<(f32, f32)> {
    let Some(first) = rows.first() else {
        return Vec::new();
    };
    let mut weeks: Vec<(i64, usize, usize)> = Vec::new();
    let (mut passes, mut total) = (0usize, 0usize);
    for r in rows {
        let week = (r.at_ms - first.at_ms).max(0) / WEEK_MS;
        passes += usize::from(passed(r.rating));
        total += 1;
        match weeks.last_mut() {
            Some(last) if last.0 == week => {
                last.1 = passes;
                last.2 = total;
            }
            _ => weeks.push((week, passes, total)),
        }
    }
    weeks
        .into_iter()
        .map(|(week, pass, total)| (week as f32, pass as f32 / total as f32))
        .collect()
}

/// Easy ratings against in-time answers, over the timed answers.
fn calibration(timed: &[(&HistoryRow, u32, u32)]) -> Option<Calibration> {
    if timed.is_empty() {
        return None;
    }
    let n = timed.len() as f32;
    let easy = timed
        .iter()
        .filter(|(r, _, _)| r.rating == Rating::Easy)
        .count() as f32;
    let in_time = timed.iter().filter(|(_, a, e)| a <= e).count() as f32;
    Some(Calibration {
        easy_share: easy / n,
        in_time_share: in_time / n,
    })
}
