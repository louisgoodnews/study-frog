#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The performance explorer on the reports screen.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_report::explore::{DayPoint, Perf};
use studyfrog_service::explore::{Explorer, Period, ReviewLine, Row, RowKind, Scope};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::explorer_ui::{ExplorerMsg, ReportTab};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn perf(reviews: u32) -> Perf {
    Perf {
        reviews,
        items: 2,
        ratings: [1, 1, reviews.saturating_sub(3), 1],
        median_ms: Some(3_000),
        total_ms: 120_000,
        last_ms: Some(1_800_000_000_000),
    }
}

fn explorer(scope: Scope) -> Explorer {
    let chart = (0..30)
        .map(|i| DayPoint {
            day: 20_000 + i,
            reviews: (i % 4) as u32,
            good: (i % 2) as u32,
        })
        .collect();
    let rows = vec![
        Row {
            label: "French".into(),
            kind: RowKind::Stack,
            target: Scope::Stack("french".into()),
            perf: perf(9),
        },
        Row {
            label: "Bonjour".into(),
            kind: RowKind::Item(ItemKind::Card),
            target: Scope::Item(Uid(5)),
            perf: perf(4),
        },
    ];
    Explorer {
        crumbs: vec![
            ("All".into(), Scope::Vault),
            ("French".into(), Scope::Stack("french".into())),
        ],
        history: vec![ReviewLine {
            at_ms: 1_800_000_000_000,
            rating: 3,
        }],
        subjects: vec![("Maths".into(), perf(5))],
        perf: perf(10),
        scope,
        chart,
        rows,
    }
}

fn model(scope: Scope) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::Loaded(Ok(Box::new(explorer(scope))))),
    );
    m
}

#[test]
fn the_screen_shows_the_numbers_the_chart_and_one_level_down() {
    let mut h = harness(model(Scope::Stack("french".into())));
    h.run_steps(2);
    for text in [
        "Performance",
        "Memory and forecast",
        "Reviews",
        "Items reviewed",
        "Good or easy",
        "Median time to answer",
        "French",
        "Bonjour",
        "By subject",
        "Maths",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    assert!(
        h.query_all_by_label_contains("reviews on").count() >= 1,
        "the chart has a text alternative"
    );
    assert!(h.query_by_label("Period: Last 7 days").is_some());
    assert_all_named(&h, "performance");
}

#[test]
fn clicking_a_row_or_a_crumb_or_a_period_asks_for_the_numbers() {
    let mut h = harness(model(Scope::Vault));
    h.run_steps(2);
    h.get_all_by_label("French").last().unwrap().click();
    h.run_steps(2);
    assert!(h.state().1.contains(&Effect::Explore {
        scope: Scope::Stack("french".into()),
        period: Period::All
    }));
    h.get_by_label("Period: Last 30 days").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.explorer.period, Period::Days(30));
    assert!(h.state().1.contains(&Effect::Explore {
        scope: Scope::Stack("french".into()),
        period: Period::Days(30)
    }));
}

#[test]
fn an_item_shows_its_reviews_and_a_way_to_open_it() {
    let mut h = harness(model(Scope::Item(Uid(5))));
    h.run_steps(2);
    assert!(h.query_by_label("Every review").is_some());
    assert!(h.query_all_by_label_contains("Good").count() >= 1);
    h.get_by_label("Open the item").click();
    h.run_steps(2);
    assert!(h.state().1.contains(&Effect::LoadDocument(Uid(5))));
}

#[test]
fn the_other_tab_still_shows_the_memory_reports() {
    let mut m = model(Scope::Vault);
    apply(&mut m, Msg::Explorer(ExplorerMsg::Tab(ReportTab::Memory)));
    let mut h = harness(m);
    h.run_steps(2);
    assert!(h.query_by_label("Computing the reports…").is_some());
    assert!(
        h.query_by_label("Reviews").is_none(),
        "the performance tiles are on the other tab"
    );
}

#[test]
fn a_failure_is_reported_and_the_last_numbers_stay() {
    let mut m = model(Scope::Vault);
    apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::Loaded(Err("disk".into()))),
    );
    assert!(m.state.notice.as_ref().unwrap().text.contains("disk"));
    assert!(m.state.explorer.data.is_some());
}
