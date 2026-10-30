#![allow(missing_docs, clippy::unwrap_used, clippy::expect_used)] // criterion_group! generates an undocumented fn

//! Layout step and projection for 5,000 nodes and about 10,000 edges (TASK-0043).

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use studyfrog_graph::layout::{Layout, layout_new, layout_step};
use studyfrog_graph::params::params_default;
use studyfrog_graph::view::{project, view_fit};

const NODES: usize = 5_000;
const EDGES: usize = 10_000;

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *seed >> 33
}

/// A notes-like graph: mostly local links with some hubs.
pub fn edges(n: usize, m: usize) -> Vec<(u32, u32)> {
    let mut seed = 42;
    (0..m)
        .map(|_| {
            let a = (lcg(&mut seed) % n as u64) as u32;
            let local = (lcg(&mut seed) % 40) as u32;
            let b = if lcg(&mut seed).is_multiple_of(5) {
                (lcg(&mut seed) % 20) as u32
            } else {
                (a + 1 + local) % n as u32
            };
            (a, if b == a { (b + 1) % n as u32 } else { b })
        })
        .collect()
}

fn settled(n: usize, e: &[(u32, u32)]) -> Layout {
    let p = params_default();
    let mut l = layout_new(n, 1, &p);
    for _ in 0..150 {
        layout_step(&mut l, e, &p);
    }
    l
}

fn bench_layout(c: &mut Criterion) {
    let p = params_default();
    let e = edges(NODES, EDGES);
    let mut l = settled(NODES, &e);
    let mut group = c.benchmark_group("graph_5k");
    group.bench_function("layout_step", |b| {
        b.iter(|| black_box(layout_step(&mut l, &e, &p)))
    });
    let (mut sx, mut sy) = (vec![0.0; NODES], vec![0.0; NODES]);
    let view = view_fit(&l, 1200.0, 800.0);
    group.bench_function("project", |b| {
        b.iter(|| project(black_box(&l), &view, 600.0, 400.0, &mut sx, &mut sy))
    });
    group.finish();
}

criterion_group!(benches, bench_layout);
criterion_main!(benches);
