#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! A report bundle with numbers in it, for the report screens.

use studyfrog_core::event::Difficulty;
use studyfrog_core::uid::Uid;
use studyfrog_report::backtest::BacktestError;
use studyfrog_report::latency::LatencyStats;
use studyfrog_report::retention::RetentionBucket;
use studyfrog_service::reports::{BacktestReport, MarkLatency, ReportBundle};

pub fn stats(p50: u32) -> LatencyStats {
    LatencyStats {
        n: 40,
        p50,
        p90: p50 * 2,
        p95: p50 * 3,
        max: p50 * 4,
    }
}

pub fn group(name: &str, percent: u8, items: usize) -> studyfrog_service::fam_report::FamGroup {
    studyfrog_service::fam_report::FamGroup {
        name: name.into(),
        folder: name.to_lowercase(),
        percent: Some(percent),
        items,
        least: vec![studyfrog_service::fam_report::FamRow {
            uid: Uid(7),
            title: "Subjonctif".into(),
            percent: 12,
            drivers: "1 rating(s), the last again 3 day(s) ago".into(),
        }],
    }
}

pub fn no_replay() -> studyfrog_report::replay::ReplayReport {
    studyfrog_report::replay::replay(&[], &studyfrog_srs::params::params_default())
}

pub fn bundle() -> ReportBundle {
    let bucket = |from_days, to_days, reviews, recalled| RetentionBucket {
        from_days,
        to_days,
        reviews,
        recalled,
    };
    ReportBundle {
        replay: no_replay(),
        runs: vec![],
        familiarity: studyfrog_service::fam_report::FamReport {
            spread: [0, 0, 1, 0, 0, 0, 0, 2, 0, 1],
            unscored: 2,
            stacks: vec![group("French", 63, 4)],
            subjects: vec![group("Languages", 63, 4)],
        },
        reviews: 120,
        retention: vec![
            bucket(1, 1, 45, 39),
            bucket(2, 2, 0, 0),
            bucket(3, 4, 30, 24),
            bucket(91, u32::MAX, 4, 4),
        ],
        latency: Some(stats(3_000)),
        histogram: vec![5, 10, 12, 8, 3, 2, 0, 0, 0],
        by_mark: vec![MarkLatency {
            mark: Some(Difficulty::Difficult),
            stats: stats(9_000),
        }],
        forecast: (0..30)
            .map(|i| if i < 7 { 10.0 - i as f32 } else { 2.0 })
            .collect(),
        backtest: BacktestReport {
            error: Some(BacktestError {
                days: 28,
                mae_per_day: 2.0,
                weekly_error: 0.1,
                bias: -0.08,
            }),
            simulated: true,
        },
        calibration: studyfrog_report::calibration::calibration_report(&[], &[]),
        analytics: Default::default(),
    }
}

/// The analytics of a simulated learner (200 days, 10 new items a day, seed 5).
pub fn analytics() -> studyfrog_service::analytics::Analytics {
    use studyfrog_report::retention_survival::{SurvivalReview, survival_report};
    let p = studyfrog_srs::params::params_default();
    let history = studyfrog_report::simulate::simulate_history(
        &studyfrog_report::simulate::SimConfig {
            seed: 5,
            days: 200,
            new_per_day: 10,
            miss_percent: 20,
        },
        &p,
    );
    let mut last = std::collections::HashMap::new();
    let reviews: Vec<SurvivalReview> = history
        .iter()
        .map(|r| SurvivalReview {
            item: r.item,
            rating: r.rating,
            elapsed_days: last
                .insert(r.item, r.day)
                .map_or(0.0, |d| (r.day - d) as f32),
        })
        .collect();
    studyfrog_service::analytics::Analytics {
        survival: survival_report(&reviews, &p),
    }
}
