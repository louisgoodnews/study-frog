#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The reports screen.

mod common;

use common::views::{assert_all_named, harness, open_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::Difficulty;
use studyfrog_core::uid::Uid;
use studyfrog_report::backtest::BacktestError;
use studyfrog_report::latency::LatencyStats;
use studyfrog_report::retention::RetentionBucket;
use studyfrog_service::reports::{BacktestReport, MarkLatency, ReportBundle};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

fn stats(p50: u32) -> LatencyStats {
    LatencyStats {
        n: 40,
        p50,
        p90: p50 * 2,
        p95: p50 * 3,
        max: p50 * 4,
    }
}

fn group(name: &str, percent: u8, items: usize) -> studyfrog_service::fam_report::FamGroup {
    studyfrog_service::fam_report::FamGroup {
        name: name.into(),
        percent: Some(percent),
        items,
        least: vec![studyfrog_service::fam_report::FamRow {
            uid: Uid(7),
            title: "Subjonctif".into(),
            percent: 12,
            drivers: "1 rating(s), the last again 3 day(s) ago".into(),
        }],
    }
}

fn no_replay() -> studyfrog_report::replay::ReplayReport {
    studyfrog_report::replay::replay(&[], &studyfrog_srs::params::params_default())
}

fn bundle() -> ReportBundle {
    let bucket = |from_days, to_days, reviews, recalled| RetentionBucket {
        from_days,
        to_days,
        reviews,
        recalled,
    };
    ReportBundle {
        replay: no_replay(),
        familiarity: studyfrog_service::fam_report::FamReport {
            spread: [0, 0, 1, 0, 0, 0, 0, 2, 0, 1],
            unscored: 2,
            stacks: vec![group("French", 63, 4)],
            subjects: vec![group("Languages", 63, 4)],
        },
        reviews: 120,
        retention: vec![
            bucket(1, 1, 45, 39),
            bucket(2, 2, 0, 0),
            bucket(3, 4, 30, 24),
            bucket(91, u32::MAX, 4, 4),
        ],
        latency: Some(stats(3_000)),
        histogram: vec![5, 10, 12, 8, 3, 2, 0, 0, 0],
        by_mark: vec![MarkLatency {
            mark: Some(Difficulty::Difficult),
            stats: stats(9_000),
        }],
        forecast: (0..30)
            .map(|i| if i < 7 { 10.0 - i as f32 } else { 2.0 })
            .collect(),
        backtest: BacktestReport {
            error: Some(BacktestError {
                days: 28,
                mae_per_day: 2.0,
                weekly_error: 0.1,
                bias: -0.08,
            }),
            simulated: true,
        },
    }
}

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
