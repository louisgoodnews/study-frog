#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Familiarity and Runs tabs of the reports screen.

mod common;

use common::reports::bundle;
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_ui::state::Screen;

fn run_row(
    uid: u128,
    mode: studyfrog_core::run::RunMode,
    pacing: studyfrog_core::pacing::Pacing,
    used_ms: u32,
    rated: u32,
) -> studyfrog_service::run_history::RunSummary {
    studyfrog_service::run_history::RunSummary {
        uid: Uid(uid),
        mode,
        pacing,
        state: "finished".into(),
        started_ms: 1_800_000_000_000,
        rated,
        skipped: 0,
        used_ms,
        exam: None,
    }
}

#[test]
fn the_familiarity_tab_shows_the_spread_the_groups_and_why_each_item_is_listed() {
    let mut m = common::views::open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = studyfrog_ui::explorer_ui::ReportTab::Familiarity;
    m.state.reports = Some(bundle());
    let mut h = harness(m);
    h.run_steps(4);
    for text in [
        "By stack",
        "By subject",
        "How well you know your items",
        "0-9 %",
        "90-100 %",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert!(
        h.query_by_label_contains("French: 63 % (4 item(s))")
            .is_some()
    );
    assert!(
        h.query_all_by_label_contains("the last again 3 day(s) ago")
            .count()
            >= 1
    );
    assert!(
        h.query_by_label_contains("2 item(s) have no history yet")
            .is_some()
    );
    assert_all_named(&h, "familiarity report");
}

#[test]
fn the_runs_tab_lists_the_history_and_filters_by_mode() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = studyfrog_ui::explorer_ui::ReportTab::Runs;
    let mut bundle = bundle();
    bundle.runs = vec![
        run_row(
            1,
            studyfrog_core::run::RunMode::Flip,
            studyfrog_core::pacing::Pacing::Free,
            60_000,
            6,
        ),
        run_row(
            2,
            studyfrog_core::run::RunMode::Answer,
            studyfrog_core::pacing::Pacing::Countdown(300),
            290_000,
            20,
        ),
    ];
    m.state.reports = Some(bundle);
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Countdown of 05:00, used 04:50")
            .is_some()
    );
    assert!(h.query_by_label_contains("6.0 answers a minute").is_some());
    assert_all_named(&h, "runs report");
    h.get_by_label("Type answers").click();
    h.run_steps(3);
    assert!(h.query_by_label_contains("Countdown of 05:00").is_some());
    assert!(
        h.query_by_label_contains("6.0 answers a minute").is_none(),
        "the flip run is filtered out"
    );
    h.get_by_label("All runs").click();
    h.run_steps(3);
    assert!(h.query_by_label_contains("6.0 answers a minute").is_some());
}
