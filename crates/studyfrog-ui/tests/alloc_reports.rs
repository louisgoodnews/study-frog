#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Allocations per frame in the reports render loop: whole app frames on the Reports screen with the
//! memory reports built. Measured with the dhat heap profiler, installed as this test binary's global
//! allocator (a safe API). This file holds exactly one test, because dhat allows one profiler per process.

mod common;

use common::reports::bundle;
use common::views::{harness, open_model};
use studyfrog_ui::explorer_ui::ReportTab;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::{Msg, MsgReports};
use studyfrog_ui::state::Screen;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const WARM_UP: usize = 3;
const FRAMES: usize = 50;
/// Allocations per frame measured when TASK-0285 was done (1275.4), plus about 11%. egui allocates
/// within each frame, so zero is not reachable; a rise above the budget is a regression to review.
const BUDGET_PER_FRAME: f64 = 1420.0;

#[test]
fn reports_frames_allocation_count() {
    let _profiler = dhat::Profiler::builder().testing().build();
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Memory;
    apply(
        &mut m,
        Msg::Reports(MsgReports::ReportsBuilt(Ok(Box::new(bundle())))),
    );
    let mut h = harness(m);
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
        "reports: {} allocations in {FRAMES} frames ({per_frame:.1} per frame)",
        after - before
    );
    assert!(
        per_frame <= BUDGET_PER_FRAME,
        "{per_frame:.1} allocations per frame, budget {BUDGET_PER_FRAME}"
    );
}
