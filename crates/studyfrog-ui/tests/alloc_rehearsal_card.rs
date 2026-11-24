#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Allocations per frame in the rehearsal card's render loop: whole app frames with a card on screen.
//! Measured with the dhat heap profiler, installed as this test binary's global allocator (a safe API).
//! This file holds exactly one test, because dhat allows one profiler per process.

mod common;

use common::views::{harness, running_model};
use studyfrog_core::run::RunMode;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const WARM_UP: usize = 3;
const FRAMES: usize = 50;
/// Allocations per frame measured when TASK-0285 was done (927.2), plus about 11%. egui allocates
/// within each frame, so zero is not reachable; a rise above the budget is a regression to review.
const BUDGET_PER_FRAME: f64 = 1030.0;

#[test]
fn rehearsal_card_frames_allocation_count() {
    let _profiler = dhat::Profiler::builder().testing().build();
    let mut h = harness(running_model(RunMode::Flip));
    h.run_steps(WARM_UP);

    let c0 = dhat::HeapStats::get().total_blocks;
    let canary = vec![1_u8; 64];
    assert!(
        dhat::HeapStats::get().total_blocks > c0 && canary.len() == 64,
        "dhat must count allocations"
    );
    drop(canary);

    let before = dhat::HeapStats::get().total_blocks;
    h.run_steps(FRAMES);
    let after = dhat::HeapStats::get().total_blocks;
    let per_frame = (after - before) as f64 / FRAMES as f64;
    eprintln!(
        "rehearsal card: {} allocations in {FRAMES} frames ({per_frame:.1} per frame)",
        after - before
    );
    assert!(
        per_frame <= BUDGET_PER_FRAME,
        "{per_frame:.1} allocations per frame, budget {BUDGET_PER_FRAME}"
    );
}
