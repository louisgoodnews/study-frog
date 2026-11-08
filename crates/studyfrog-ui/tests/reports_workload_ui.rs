#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Workload tab, its choices, and the exam pacer on the stack page (STORY-0105).

mod common;

use common::analytics::analytics;
use common::reports::bundle;
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::explorer_ui::{ExplorerMsg, ReportTab};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};

fn with_reports(mut m: studyfrog_ui::model::Model) -> studyfrog_ui::model::Model {
    let mut b = bundle();
    b.analytics = Box::new(analytics());
    m.state.reports = Some(b);
    m
}

fn on_workload() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Workload;
    with_reports(m)
}

const PACER: [&str; 4] = [
    "French: exam on 2027-01-31, in 20 day(s).",
    "Start 2 new card(s) a day for 17 day(s) (34 never seen), then keep the last 3 day(s) for consolidation.",
    "About 12 review(s) and 6.2 minute(s) a day; 5 card(s) would fall under 90 % without another review, about 4 consolidation review(s) at the end.",
    "On track: 93 % of the cards would be at 90 % recall on the day.",
];

#[test]
fn thirty_days_by_day_then_by_week_with_stages_and_band() {
    let mut h = harness(on_workload());
    h.run_steps(4);
    for text in [
        "About 300 reviews and 150 minutes of study in the next 30 days.",
        "Today",
        "In 6 days",
        "Days 7-13",
        "Days 28-29",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_eq!(
        h.query_all_by_label("10 reviews").count(),
        7,
        "one per day of the first week"
    );
    for text in PACER {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_all_named(&h, "workload report");
}

#[test]
fn the_horizon_and_the_unit_can_be_changed() {
    let mut h = harness(on_workload());
    h.run_steps(4);
    h.get_by_label("365 days").click();
    h.run_steps(2);
    h.get_by_label("Study minutes").click();
    h.run_steps(3);
    let e = &h.state().0.state.explorer;
    assert_eq!((e.horizon, e.minutes), (365, true));
    assert!(
        h.query_by_label("About 3650 reviews and 1825 minutes of study in the next 365 days.")
            .is_some()
    );
    assert!(h.query_by_label("Days 336-363").is_some());
    assert!(
        h.query_by_label("In 364 days").is_some(),
        "the last day on its own"
    );
    assert!(
        h.query_all_by_label("140 min").count() >= 1,
        "four weeks of 5 minutes"
    );
    let mut m = on_workload();
    apply(&mut m, Msg::Explorer(ExplorerMsg::Horizon(90)));
    assert_eq!(m.state.explorer.horizon, 90);
}

#[test]
fn the_stack_page_asks_for_the_reports_and_shows_its_exam_pacer() {
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:summary".into()];
    let fx = apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    assert!(
        fx.contains(&studyfrog_ui::effect::Effect::BuildReports),
        "{fx:?}"
    );
    // Opening the Summary builds the reports for its pacer; the Reports tab shows the same card.
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:reports".into()];
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let mut h = harness(with_reports(m));
    h.run_steps(4);
    for text in PACER {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
}
