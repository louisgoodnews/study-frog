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

fn leeches(c: &mut Criterion) {
    use studyfrog_core::review::Rating;
    use studyfrog_report::confusion::{PairInput, confusion_pairs, pair_rules_default, token_set};
    use studyfrog_report::leech::{LeechInput, find_leeches, leech_rules_default};
    let mut by: HashMap<u64, Vec<Rating>> = HashMap::new();
    for r in history() {
        by.entry(r.item).or_default().push(r.rating);
    }
    let items: Vec<LeechInput> = by
        .into_iter()
        .map(|(item, ratings)| LeechInput {
            item,
            group: (item % 50) as u32,
            total_ms: ratings.len() as u64 * (3_000 + item % 7 * 1_000),
            stability: Some((item % 40) as f32),
            ratings,
        })
        .collect();
    println!("{} items", items.len());
    c.bench_function("leeches of every item", |b| {
        b.iter(|| find_leeches(black_box(&items), &leech_rules_default()));
    });
    let words = [
        "ionic bond electrons",
        "covalent bond electrons",
        "molar mass water",
        "cell unit life",
    ];
    let pairs: Vec<PairInput> = (0..400_u64)
        .map(|i| PairInput {
            item: i,
            group: (i % 10) as u32,
            tokens: token_set(&format!("{} {i}", words[(i % 4) as usize])),
            tags: vec![(i % 13) as u32],
            links: vec![],
            lapse_days: {
                let mut d: Vec<i64> = (0..6).map(|k| (i as i64 * 7 + k * 11) % 120).collect();
                d.sort_unstable();
                d
            },
        })
        .collect();
    c.bench_function("confusion pairs of 400 lapsing items", |b| {
        b.iter(|| confusion_pairs(black_box(&pairs), &pair_rules_default()));
    });
}

fn deck(n: i64) -> Vec<studyfrog_report::sim_engine::SimCard> {
    use studyfrog_core::memory::MemState;
    use studyfrog_report::sim_engine::{SimCard, SimMemory};
    (0..n)
        .map(|i| SimCard {
            memory: (i % 5 != 0).then_some(SimMemory::Fsrs(MemState {
                stability: 1.0 + (i % 120) as f32,
                difficulty: 5.0,
            })),
            last_day: -(i % 30),
            due_day: i % 40,
            relearning: false,
            sm2: false,
            pace: 0,
        })
        .collect()
}

fn sim_setup(days: usize, trajectories: u32) -> studyfrog_report::sim_engine::SimSetup {
    use studyfrog_report::sim_engine::{RatingMix, SimSetup, pace_default};
    SimSetup {
        today: 0,
        days,
        trajectories,
        seed: 1,
        miss_probability: 0.1,
        new_per_day: 10.0,
        new_days: days,
        mix: RatingMix {
            first_again: 0.2,
            hard: 0.1,
            good: 0.8,
        },
        paces: vec![pace_default()],
    }
}

fn forecasts(c: &mut Criterion) {
    use studyfrog_report::forecast_sim::simulate_forecast;
    let (cards, p) = (deck(5_000), params_default());
    for (days, n) in [(90, 16), (365, 16)] {
        let s = sim_setup(days, n);
        c.bench_function(
            &format!("forecast of 5,000 cards, {days} days, {n} futures"),
            |b| {
                b.iter(|| simulate_forecast(black_box(&cards), &s, &p));
            },
        );
    }
}

fn retention_curve(c: &mut Criterion) {
    use studyfrog_report::cmrr::cmrr;
    let (cards, p) = (deck(5_000), params_default());
    let s = sim_setup(180, 4);
    c.bench_function(
        "cmrr of 5,000 cards, 12 targets, 180 days, 4 futures",
        |b| {
            b.iter(|| cmrr(black_box(&cards), &s, &p));
        },
    );
}

criterion_group!(
    benches,
    survival,
    habits,
    leeches,
    forecasts,
    retention_curve
);
criterion_main!(benches);
