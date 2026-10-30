//! Report computation on a history of about 100,000 reviews (budgets in docs/BENCHMARKING.md).

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use studyfrog_report::backtest::{DatedReview, backtest};
use studyfrog_report::latency::{histogram, latency_stats};
use studyfrog_report::retention::{ReviewPoint, retention_curve};
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_srs::params::params_default;

fn reports(c: &mut Criterion) {
    let params = params_default();
    let history: Vec<DatedReview> = simulate_history(
        &SimConfig {
            seed: 1,
            days: 400,
            new_per_day: 40,
            miss_percent: 10,
        },
        &params,
    );
    let points: Vec<ReviewPoint> = history
        .iter()
        .map(|r| ReviewPoint {
            rating: r.rating,
            elapsed_days: (r.day % 40 + 1) as f32,
        })
        .collect();
    let samples: Vec<u32> = history
        .iter()
        .map(|r| (r.day as u32 * 7919 + r.item as u32 * 31) % 40_000)
        .collect();
    println!("{} reviews", history.len());
    c.bench_function("retention curve over the history", |b| {
        b.iter(|| retention_curve(black_box(&points)))
    });
    c.bench_function(
        "latency stats and histogram over one sample per review",
        |b| {
            b.iter(|| {
                let mut copy = samples.clone();
                (latency_stats(&mut copy), histogram(black_box(&samples)))
            });
        },
    );
    c.bench_function("replay, estimate, forecast and compare (backtest)", |b| {
        b.iter(|| backtest(black_box(&history), 340, 28, &params));
    });
}

criterion_group!(benches, reports);
criterion_main!(benches);
