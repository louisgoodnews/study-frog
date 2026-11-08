#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Analytics with numbers in them (EPIC-034), for the report screens.

use studyfrog_core::uid::Uid;

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
        chronotype: chronotype(),
        fatigue: studyfrog_report::fatigue::fatigue_report(&tiring()),
        leeches: vec![leech_row()],
        pairs: vec![pair_row()],
        plan: plan(),
    }
}

/// Every day 10 reviews (1 new, 1 learning, 2 young, 6 mature; 8 to 12) and 5 minutes (4 to 6); French has an
/// exam on 2027-01-31 (day 20 849), 20 days ahead.
pub fn plan() -> studyfrog_service::analytics_plan::Plan {
    use studyfrog_report::exam_pacer::ExamPace;
    use studyfrog_report::forecast_sim::DayLoad;
    let day = DayLoad {
        new: 1.0,
        learning: 1.0,
        young: 2.0,
        mature: 6.0,
        p10: 8.0,
        p50: 10.0,
        p90: 12.0,
        minutes: 5.0,
        minutes_p10: 4.0,
        minutes_p90: 6.0,
    };
    studyfrog_service::analytics_plan::Plan {
        forecast: vec![day; 365],
        miss_probability: 0.1,
        new_per_day: 1.0,
        mix: None,
        pacers: vec![studyfrog_service::analytics_plan::StackPace {
            uid: Uid(1),
            folder: "french".into(),
            title: "French".into(),
            exam_day: 20_849,
            pace: ExamPace {
                days_left: 20,
                study_days: 17,
                unseen_items: 34,
                volatile_items: 5,
                required_new_per_day: 2,
                reviews_per_day: 12.4,
                minutes_per_day: 6.2,
                consolidation_reviews: 4.0,
                ready_share: 0.93,
                on_track: true,
            },
        }],
    }
}

/// "Subjonctif" in French: five lapses, a streak of two, four times the usual time.
pub fn leech_row() -> studyfrog_service::analytics_leech::LeechRow {
    use studyfrog_report::leech::{LeechItem, LeechReason};
    studyfrog_service::analytics_leech::LeechRow {
        uid: Uid(7),
        title: "Subjonctif".into(),
        folder: "french".into(),
        leech: LeechItem {
            item: 0,
            group: 0,
            lapses: 5,
            streak: 2,
            reviews: 9,
            time_burden: Some(4.0),
            efficiency: Some(0.5),
            reasons: vec![LeechReason::RepeatedLapses, LeechReason::HighLatencyBurden],
            score: 2.5,
        },
    }
}

/// "Ionic bond" and "Covalent bond" in Chemistry.
pub fn pair_row() -> studyfrog_service::analytics_leech::PairRow {
    studyfrog_service::analytics_leech::PairRow {
        a: (Uid(10), "Ionic bond".into()),
        b: (Uid(11), "Covalent bond".into()),
        folders: ("chemistry".into(), "chemistry".into()),
        pair: studyfrog_report::confusion::ConfusionPair {
            a: 0,
            b: 1,
            similarity: 0.4,
            co_lapses: 3,
            shared_tags: 1,
            linked: false,
            same_group: true,
            score: 1.95,
        },
    }
}

/// 20 reviews at 09:00, 10:00 (one Again each), 20:00 and 21:00 (ten Again each), Monday 2027-01-11 UTC.
pub fn chronotype() -> studyfrog_report::chronotype::ChronotypeReport {
    use studyfrog_report::chronotype::{TimedLatency, TimedReview};
    const MONDAY: i64 = 1_799_625_600_000;
    let mut reviews = Vec::new();
    let mut latencies = Vec::new();
    for (hour, again, ms) in [
        (9, 1, 3_000),
        (10, 1, 3_000),
        (20, 10, 6_000),
        (21, 10, 6_000),
    ] {
        for k in 0..20 {
            let at_ms = MONDAY + hour * 3_600_000 + k * 60_000;
            let rating = if k < again {
                studyfrog_core::review::Rating::Again
            } else {
                studyfrog_core::review::Rating::Good
            };
            reviews.push(TimedReview {
                at_ms,
                offset_minutes: 0,
                rating,
            });
            latencies.push(TimedLatency {
                at_ms,
                offset_minutes: 0,
                ms,
            });
        }
    }
    studyfrog_report::chronotype::chronotype(&reviews, &latencies, 20)
}

/// Three runs of 30 minutes, one item a minute: steady for 15 minutes (one Again in ten), then slower and every
/// other one forgotten.
pub fn tiring() -> Vec<studyfrog_report::session_log::SessionSample> {
    let mut out = Vec::new();
    for run in 1..=3_u128 {
        for minute in 0..30_u32 {
            let (latency, again) = if minute < 15 {
                (3_000, minute % 10 == 9)
            } else {
                (5_000, minute % 2 == 0)
            };
            out.push(studyfrog_report::session_log::SessionSample {
                run: Uid(run),
                item: Uid(u128::from(minute) + 1),
                position: minute + 1,
                shown_t_ms: minute * 60_000,
                active_ms: minute * 60_000,
                latency_ms: Some(latency),
                item_ms: latency + 1_000,
                rating: if again {
                    studyfrog_core::review::Rating::Again
                } else {
                    studyfrog_core::review::Rating::Good
                },
            });
        }
    }
    out
}
