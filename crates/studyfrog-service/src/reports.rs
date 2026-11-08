//! The reports: retention, answer latency and the review-load forecast with its backtest.

use std::collections::HashMap;

use studyfrog_core::event::Difficulty;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::backtest::{BacktestError, DatedReview, backtest, estimate_model};
use studyfrog_report::forecast::{ScheduledItem, forecast};
use studyfrog_report::latency::{LatencyStats, flip_samples, histogram, latency_stats};
use studyfrog_report::retention::{RetentionBucket, ReviewPoint, retention_curve};
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_srs::params::Params;
use studyfrog_store::reports::{HistoryRow, load_all_events, load_review_history, load_scheduled};

use crate::session::{ServiceError, VaultSession, current_settings};
use crate::settings::params_for;

const MS_PER_DAY: i64 = 86_400_000;
/// Days the forecast covers.
pub const FORECAST_DAYS: usize = 30;
/// Days the backtest forecasts and compares.
pub const BACKTEST_DAYS: usize = 28;
/// Days of real history needed before the backtest uses it instead of a simulated one.
pub const MIN_REAL_HISTORY_DAYS: i64 = 56;

/// How good the forecast has been.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BacktestReport {
    /// The measured error, if it could be measured.
    pub error: Option<BacktestError>,
    /// True when the history was simulated because the real one is too short.
    pub simulated: bool,
}

/// Answer times of items the user marked one way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkLatency {
    /// The mark (`None`: not marked).
    pub mark: Option<Difficulty>,
    /// Percentiles.
    pub stats: LatencyStats,
}

/// Everything the reports screen shows.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportBundle {
    /// Reviews in the history.
    pub reviews: usize,
    /// Retention per gap bucket.
    pub retention: Vec<RetentionBucket>,
    /// Time to reveal an answer, all items.
    pub latency: Option<LatencyStats>,
    /// Counts per bin of the latency histogram.
    pub histogram: Vec<u32>,
    /// The same by the user's difficulty mark.
    pub by_mark: Vec<MarkLatency>,
    /// Expected reviews on each of the next days, starting today.
    pub forecast: Vec<f32>,
    /// How good such a forecast has been.
    pub backtest: BacktestReport,
    /// How familiar the user is with the items.
    pub familiarity: crate::fam_report::FamReport,
    /// Whether familiarity improves the predictions, replayed on the history.
    pub replay: studyfrog_report::replay::ReplayReport,
    /// The latest runs with their mode, pacing and time.
    pub runs: Vec<crate::run_history::RunSummary>,
    /// Metacognitive self-assessment calibration against measured performance.
    pub calibration: studyfrog_report::calibration::CalibrationReport,
    /// The behavioural analytics (EPIC-034).
    pub analytics: crate::analytics::Analytics,
}

fn day_of(ms: i64, offset_minutes: i32) -> i64 {
    (ms + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY)
}

fn dated(rows: &[HistoryRow], offset: i32) -> Vec<DatedReview> {
    let mut ids: HashMap<Uid, u64> = HashMap::new();
    rows.iter()
        .map(|r| {
            let next = ids.len() as u64;
            DatedReview {
                item: *ids.entry(r.item).or_insert(next),
                day: day_of(r.at_ms, offset),
                rating: r.rating,
            }
        })
        .collect()
}

fn backtest_report(history: &[DatedReview], today: i64, params: &Params) -> BacktestReport {
    let first = history.first().map(|r| r.day);
    if first.is_some_and(|f| today - f >= MIN_REAL_HISTORY_DAYS) {
        let error = backtest(history, today - BACKTEST_DAYS as i64, BACKTEST_DAYS, params);
        if error.is_some() {
            return BacktestReport {
                error,
                simulated: false,
            };
        }
    }
    let sim = simulate_history(
        &SimConfig {
            seed: 1,
            days: 150,
            new_per_day: 12,
            miss_percent: 10,
        },
        params,
    );
    BacktestReport {
        error: backtest(&sim, 119, BACKTEST_DAYS, params),
        simulated: true,
    }
}

fn latency_by_mark(samples: &[studyfrog_report::latency::FlipSample]) -> Vec<MarkLatency> {
    [
        None,
        Some(Difficulty::Easy),
        Some(Difficulty::Medium),
        Some(Difficulty::Difficult),
    ]
    .into_iter()
    .filter_map(|mark| {
        let mut ms: Vec<u32> = samples
            .iter()
            .filter(|s| s.mark == mark)
            .map(|s| s.ms)
            .collect();
        latency_stats(&mut ms).map(|stats| MarkLatency { mark, stats })
    })
    .collect()
}

/// Compute all reports at `now` (`offset_minutes` is the local UTC offset).
pub fn build_reports(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<ReportBundle, ServiceError> {
    let store = ServiceError::Store;
    let params = params_for(&current_settings(session), None);
    let rows = load_review_history(&session.conn).map_err(store)?;
    let points: Vec<ReviewPoint> = rows
        .iter()
        .map(|r| ReviewPoint {
            rating: r.rating,
            elapsed_days: r.elapsed_days,
        })
        .collect();
    let history = dated(&rows, offset_minutes);
    let today = day_of(now.0, offset_minutes);

    let samples = flip_samples(&load_all_events(&session.conn).map_err(ServiceError::Store)?);
    let mut all: Vec<u32> = samples.iter().map(|s| s.ms).collect();
    let bins = histogram(&all);
    let items: Vec<ScheduledItem> = load_scheduled(&session.conn)
        .map_err(ServiceError::Store)?
        .into_iter()
        .map(|(due_ms, interval)| ScheduledItem {
            due_day: day_of(due_ms, offset_minutes),
            interval_days: interval,
        })
        .collect();
    let model = estimate_model(&history, today);
    Ok(ReportBundle {
        reviews: rows.len(),
        retention: retention_curve(&points),
        latency: latency_stats(&mut all),
        histogram: bins,
        by_mark: latency_by_mark(&samples),
        forecast: forecast(&items, &model, today, FORECAST_DAYS),
        backtest: backtest_report(&history, today, &params),
        familiarity: crate::fam_report::familiarity_report(&session.conn, now).map_err(store)?,
        replay: crate::fam_replay::replay_history(&session.conn, now, &params).map_err(store)?,
        runs: crate::run_history::run_history(&session.conn).map_err(store)?,
        calibration: crate::calibration_report::build_calibration_report(
            &session.conn,
            &rows,
            now,
            offset_minutes,
        )
        .map_err(store)?,
        analytics: crate::analytics::analytics_of(&rows, &params),
    })
}
