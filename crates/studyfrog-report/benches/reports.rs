//! Report computation on a history of about 100,000 reviews (budgets in docs/BENCHMARKING.md).

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_report::backtest::{DatedReview, backtest};
use studyfrog_report::familiarity::{FamContext, FamEvent, fam_params_default, familiarity};
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

/// 10 000 items with 20 ratings and 5 views each (budget: all of them in under 20 ms).
fn familiarity_of_10k_items(c: &mut Criterion) {
    let items: Vec<Vec<FamEvent>> = (0..10_000_i64)
        .map(|i| {
            let mut events: Vec<FamEvent> = (0..20_i64)
                .map(|k| FamEvent::Rating {
                    at: UnixMs((i % 30 + k * 17) * 86_400_000),
                    rating: [Rating::Good, Rating::Easy, Rating::Hard, Rating::Again]
                        [((i + k) % 4) as usize],
                    answer_ms: Some(2_000 + (k as u32) * 100),
                    expected_ms: Some(3_000),
                })
                .collect();
            events.extend((0..5_i64).map(|k| FamEvent::View {
                at: UnixMs((i % 30 + k * 60) * 86_400_000 + 7_200_000),
                active_ms: 4_000,
                expected_ms: Some(6_000),
            }));
            events
        })
        .collect();
    let (p, ctx, now) = (
        fam_params_default(),
        FamContext::default(),
        UnixMs(400 * 86_400_000),
    );
    c.bench_function("familiarity of 10,000 items with 25 events each", |b| {
        b.iter(|| {
            items
                .iter()
                .filter_map(|e| familiarity(black_box(e), now, &ctx, &p))
                .map(|f| f.value)
                .sum::<f32>()
        });
    });
}

criterion_group!(benches, reports, familiarity_of_10k_items);
criterion_main!(benches);
