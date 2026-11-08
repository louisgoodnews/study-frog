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

criterion_group!(benches, survival);
criterion_main!(benches);
