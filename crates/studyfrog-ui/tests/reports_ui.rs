#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The reports screen.

mod common;

use common::reports::{bundle, no_replay};

use common::views::{assert_all_named, harness, open_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_service::reports::{BacktestReport, ReportBundle};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

#[test]
fn opening_the_screen_computes_the_reports_and_failures_are_reported() {
    let mut m = open_model();
    assert_eq!(
        apply(&mut m, Msg::Navigate(Screen::Reports)),
        [
            Effect::BuildReports,
            Effect::Explore {
                scope: Default::default(),
                period: Default::default()
            }
        ]
    );
    apply(&mut m, Msg::ReportsBuilt(Ok(bundle())));
    assert_eq!(m.state.reports.as_ref().unwrap().reviews, 120);
    apply(&mut m, Msg::ReportsBuilt(Err("disk".into())));
    assert_eq!(m.state.notice.as_ref().unwrap().kind, NoticeKind::Error);
    assert!(m.state.reports.is_some(), "the last reports stay");
}

#[test]
fn the_three_reports_show_their_numbers_as_text() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = studyfrog_ui::explorer_ui::ReportTab::Memory;
    apply(&mut m, Msg::ReportsBuilt(Ok(bundle())));
    let mut h = harness(m);
    h.run();
    for text in [
        "Retention",
        "1 day",
        "87 % of 45",
        "3-4 days",
        "80 % of 30",
        "91+ days",
        "100 % of 4",
        "Time to answer",
        "median 3.0 s, 90 % under 6.0 s, 95 % under 9.0 s (40 answers)",
        "under 1 s",
        "1-2 s",
        "Marked difficult: median 9.0 s, 90 % under 18.0 s, 95 % under 27.0 s (40 answers)",
        "Reviews ahead",
        "Today",
        "Tomorrow",
        "In 6 days",
        "Checked on a simulated history (yours is too short): weekly totals were off by 10 %, and the forecast ran 8 % low overall.",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert!(
        h.query_by_label("2 days").is_none(),
        "empty buckets are left out"
    );
    assert_all_named(&h, "reports");
}

#[test]
fn an_empty_history_says_so_and_still_draws() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = studyfrog_ui::explorer_ui::ReportTab::Memory;
    let empty = ReportBundle {
        replay: no_replay(),
        runs: vec![],
        familiarity: studyfrog_service::fam_report::FamReport {
            spread: [0; 10],
            unscored: 0,
            stacks: vec![],
            subjects: vec![],
        },
        reviews: 0,
        retention: vec![],
        latency: None,
        histogram: vec![],
        by_mark: vec![],
        forecast: vec![0.0; 30],
        backtest: BacktestReport {
            error: None,
            simulated: true,
        },
        calibration: studyfrog_report::calibration::calibration_report(&[], &[]),
        analytics: Default::default(),
    };
    apply(&mut m, Msg::ReportsBuilt(Ok(empty)));
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("There are no reviews yet. Rehearse a few items and come back.")
            .is_some()
    );
    assert!(h.query_by_label("No answer times recorded yet.").is_some());
    assert!(
        h.query_by_label("The forecast could not be checked yet.")
            .is_some()
    );
}

#[test]
fn the_executor_computes_reports_for_a_real_vault() {
    let vault = sample_vault("reports");
    let mut worker = worker_new(None);
    execute(
        &mut worker,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let msgs = execute(&mut worker, Effect::BuildReports, NOW_NS);
    let Msg::ReportsBuilt(Ok(r)) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!((r.reviews, r.forecast.len()), (0, 30));
    assert!(r.backtest.simulated && r.backtest.error.is_some());
}
