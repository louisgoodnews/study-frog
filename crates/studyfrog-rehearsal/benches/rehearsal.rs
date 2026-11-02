#![allow(missing_docs, clippy::unwrap_used, clippy::expect_used)] // criterion_group! generates an undocumented fn; benches may panic
//! Rehearsal hot paths (TASK-0035): one engine step, and queue building for 10k candidates.

use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use studyfrog_core::memory::MemState;
use studyfrog_core::plan::{ItemKind, Order, RunPlan, plan_sequential, plan_shuffled, plan_spaced};
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::{Action, Input};
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Candidate, budgets_unlimited};
use studyfrog_rehearsal::session::{Session, session_new};
use studyfrog_rehearsal::step::step;
use studyfrog_srs::params::params_default;

const ITEMS: u32 = 1_000;
const CANDIDATES: u32 = 10_000;
const NOW: UnixMs = UnixMs(1_800_000_000_000);

fn session() -> Session {
    let queue = (0..ITEMS).map(|i| Uid(u128::from(i) + 1)).collect();
    session_new(Run {
        uid: Uid(1),
        mode: RunMode::Flip,
        plan: plan_sequential(),
        queue,
        state: RunState::Idle,
        started_at: None,
    })
}

/// Start, then flip and rate every item: 2 * ITEMS + 1 steps.
fn run_through(mut s: Session) -> Session {
    let mut t = 0;
    step(
        &mut s,
        Input {
            t_ms: t,
            action: Action::Start,
        },
    )
    .unwrap();
    for _ in 0..ITEMS {
        t += 700;
        step(
            &mut s,
            Input {
                t_ms: t,
                action: Action::Flip,
            },
        )
        .unwrap();
        t += 300;
        step(
            &mut s,
            Input {
                t_ms: t,
                action: Action::Rate(Rating::Good),
            },
        )
        .unwrap();
    }
    s
}

fn lcg(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *seed >> 8
}

fn candidates(tags: &[String]) -> Vec<Candidate<'_>> {
    let mut seed = 7;
    (0..CANDIDATES)
        .map(|i| {
            let reviewed = !lcg(&mut seed).is_multiple_of(4);
            let memory = reviewed.then(|| MemState {
                stability: 1.0 + (lcg(&mut seed) % 3000) as f32 / 10.0,
                difficulty: 1.0 + (lcg(&mut seed) % 900) as f32 / 100.0,
            });
            let day = 86_400_000_i64;
            Candidate {
                uid: Uid(u128::from(i) + 1),
                kind: ItemKind::Card,
                stack: Uid(u128::from(i % 50) + 1),
                tags,
                position: i,
                memory,
                due: reviewed.then(|| UnixMs(NOW.0 + (i64::from(lcg(&mut seed) % 60) - 30) * day)),
                last_review: reviewed
                    .then(|| UnixMs(NOW.0 - i64::from(1 + lcg(&mut seed) % 90) * day)),
                priority: Default::default(),
                difficulty_hint: None,
                familiarity: None,
            }
        })
        .collect()
}

fn bench_rehearsal(c: &mut Criterion) {
    let mut group = c.benchmark_group("rehearsal");
    group.bench_function("run_through_1000_items_2001_steps", |b| {
        b.iter_batched(
            session,
            |s| black_box(run_through(s)),
            BatchSize::SmallInput,
        )
    });
    let tags: Vec<String> = Vec::new();
    let cands = candidates(&tags);
    let p = params_default();
    let plans: [(&str, RunPlan); 4] = [
        ("spaced", plan_spaced()),
        ("shuffled", plan_shuffled(1)),
        (
            "hardest_first",
            RunPlan {
                order: Order::HardestFirst,
                ..plan_sequential()
            },
        ),
        (
            "lowest_retrievability",
            RunPlan {
                order: Order::LowestRetrievability,
                ..plan_sequential()
            },
        ),
    ];
    for (name, plan) in &plans {
        group.bench_function(format!("build_queue_10k_{name}"), |b| {
            b.iter(|| build_queue(black_box(&cands), plan, NOW, &budgets_unlimited(), &p))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_rehearsal);
criterion_main!(benches);
