#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! After creation, stepping and projecting allocate nothing. Measured with the dhat heap profiler,
//! which is installed as this test binary's global allocator (a safe API; this crate has no unsafe).
//! This file holds exactly one test, because dhat allows one profiler per process.

use studyfrog_graph::layout::{layout_new, layout_step};
use studyfrog_graph::params::params_default;
use studyfrog_graph::view::{nearest, project, view_fit};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn stepping_projecting_and_picking_allocate_nothing() {
    let _profiler = dhat::Profiler::builder().testing().build();
    let p = params_default();
    let n = 2_000;
    let edges: Vec<(u32, u32)> = (0..4_000_u32)
        .map(|i| (i % n as u32, (i * 7 + 1) % n as u32))
        .filter(|(a, b)| a != b)
        .collect();
    let mut l = layout_new(n, 9, &p);
    let (mut sx, mut sy) = (vec![0.0_f32; n], vec![0.0_f32; n]);
    let view = view_fit(&l, 1000.0, 700.0);
    // Warm up once so any lazy one-time setup is out of the way.
    layout_step(&mut l, &edges, &p);

    // Canary: prove the profiler sees allocations, so "zero" below means something.
    let c0 = dhat::HeapStats::get().total_blocks;
    let canary = vec![1_u8; 64];
    assert!(
        dhat::HeapStats::get().total_blocks > c0 && canary.len() == 64,
        "dhat must count allocations"
    );
    drop(canary);

    let before = dhat::HeapStats::get().total_blocks;
    for _ in 0..50 {
        layout_step(&mut l, &edges, &p);
        project(&l, &view, 500.0, 350.0, &mut sx, &mut sy);
        let _ = nearest(&sx, &sy, 500.0, 350.0, 8.0);
    }
    let after = dhat::HeapStats::get().total_blocks;
    assert_eq!(
        after - before,
        0,
        "{} allocations in 50 frames",
        after - before
    );
}
