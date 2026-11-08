//! The analytics of EPIC-034 on a simulated history of about 100,000 reviews (budgets in
//! docs/contributing/BENCHMARKING.md).

use std::collections::HashMap;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use studyfrog_report::backtest::DatedReview;
use studyfrog_report::retention_survival::{SurvivalReview, survival_report};
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_srs::params::params_default;

fn history() -> Vec<DatedReview> {
    simulate_history(
        &SimConfig {
            seed: 1,
            days: 400,
            new_per_day: 40,
            miss_percent: 10,
        },
        &params_default(),
    )
}

fn survival(c: &mut Criterion) {
    let mut last = HashMap::new();
    let reviews: Vec<SurvivalReview> = history()
        .iter()
        .map(|r| SurvivalReview {
            item: r.item,
            rating: r.rating,
            elapsed_days: last
                .insert(r.item, r.day)
                .map_or(0.0, |d| (r.day - d) as f32),
        })
        .collect();
    println!("{} reviews", reviews.len());
    let p = params_default();
    c.bench_function("survival report (true retention, tier curves, fit)", |b| {
        b.iter(|| survival_report(black_box(&reviews), &p));
    });
}

fn habits(c: &mut Criterion) {
    use studyfrog_core::review::Rating;
    use studyfrog_core::uid::Uid;
    use studyfrog_report::chronotype::{TimedLatency, TimedReview, chronotype};
    use studyfrog_report::fatigue::fatigue_report;
    use studyfrog_report::session_log::SessionSample;
    let h = history();
    let at = |i: usize, day: i64| day * 86_400_000 + (i as i64 * 7_919) % 86_400_000;
    let reviews: Vec<TimedReview> = h
        .iter()
        .enumerate()
        .map(|(i, r)| TimedReview {
            at_ms: at(i, r.day),
            offset_minutes: 60,
            rating: r.rating,
        })
        .collect();
    let latencies: Vec<TimedLatency> = h
        .iter()
        .enumerate()
        .map(|(i, r)| TimedLatency {
            at_ms: at(i, r.day),
            offset_minutes: 60,
            ms: (i as u32 * 7_919) % 20_000,
        })
        .collect();
    c.bench_function("chronotype of every review and answer time", |b| {
        b.iter(|| chronotype(black_box(&reviews), black_box(&latencies), 20));
    });
    let samples: Vec<SessionSample> = h
        .iter()
        .enumerate()
        .map(|(i, r)| SessionSample {
            run: Uid(r.day as u128),
            item: Uid(r.item as u128),
            position: (i % 80) as u32 + 1,
            shown_t_ms: (i % 80) as u32 * 30_000,
            active_ms: (i % 80) as u32 * 30_000,
            latency_ms: Some((i as u32 * 7_919) % 20_000),
            item_ms: 5_000,
            rating: if i % 9 == 0 { Rating::Again } else { r.rating },
        })
        .collect();
    c.bench_function("fatigue report of one sample per review", |b| {
        b.iter(|| fatigue_report(black_box(&samples)));
    });
}

criterion_group!(benches, survival, habits);
criterion_main!(benches);
