#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
use eframe::egui;
use studyfrog_ui::effect::EffectEditor;
use studyfrog_ui::effect::EffectReports;

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
fn zz_debug() {
    let mut h = harness(model(Scope::Vault));
    h.set_size(egui::vec2(1000.0, 1000.0));
    h.run_steps(2);
    for n in h.get_all_by_label("French") {
        eprintln!("RECT {:?}", n.rect());
    }
    h.get_all_by_label("French").last().unwrap().click();
    h.run_steps(2);
    eprintln!("EFFECTS {:?}", h.state().1);
}
