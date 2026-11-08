#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The workload-versus-retention curve and "Apply recommended retention" (STORY-0106).

mod common;

use common::analytics::analytics;
use common::reports::bundle;
use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::explorer_ui::{ExplorerMsg, ReportTab};
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen, Selection};

fn with_reports(mut m: studyfrog_ui::model::Model) -> studyfrog_ui::model::Model {
    let mut b = bundle();
    b.analytics = Box::new(analytics());
    m.state.reports = Some(b);
    m
}

#[test]
fn the_workload_tab_shows_the_curve_and_applies_the_recommendation_to_the_vault() {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Workload;
    let mut h = harness(with_reports(m));
    h.run_steps(4);
    for text in [
        "Workload against desired retention",
        "Recommended for the vault: 0.86, the most cards remembered per minute of study (now 0.90). Below 0.86 forgetting adds more relearning than it saves.",
        "0.86 (recommended, least work)",
        "0.90 (now)",
        "0.80",
        "6.0 min/day",
        "35 reviews a day, 110 cards remembered (86 %), 22.0 per minute",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    h.get_by_label("Apply recommended retention 0.86 to the vault")
        .click();
    h.run_steps(2);
    let mut m = with_reports(open_model());
    let fx = apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::ApplyRetention(None, 860)),
    );
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::ApplyRetention(None, 860))]
    );
}

#[test]
fn a_stack_reports_tab_applies_to_its_own_settings() {
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:reports".into()];
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let mut h = harness(with_reports(m));
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Recommended for French: 0.86")
            .is_some()
    );
    h.get_by_label("Apply recommended retention 0.86 to French")
        .click();
    h.run_steps(2);
    let mut m = with_reports(open_model());
    let fx = apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::ApplyRetention(Some("french".into()), 860)),
    );
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::ApplyRetention(
            Some("french".into()),
            860
        ))]
    );
}

#[test]
fn the_result_refreshes_the_vault_and_the_reports_or_reports_the_failure() {
    let mut m = with_reports(open_model());
    let info = m.state.vault.clone().unwrap();
    let fx = apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::RetentionApplied(Ok(Box::new(info)))),
    );
    assert_eq!(fx, [Effect::BuildReports]);
    assert_eq!(m.state.notice.as_ref().unwrap().kind, NoticeKind::Info);
    let fx = apply(
        &mut m,
        Msg::Explorer(ExplorerMsg::RetentionApplied(Err("disk full".into()))),
    );
    assert!(fx.is_empty());
    let n = m.state.notice.as_ref().unwrap();
    assert!(n.kind == NoticeKind::Error && n.text.contains("disk full"));
}

#[test]
fn applying_through_the_worker_writes_the_stack_file() {
    use common::flow::{opened, send};
    let (v, mut model, mut worker) = opened("cmrr-apply");
    send(
        &mut model,
        &mut worker,
        Msg::Explorer(ExplorerMsg::ApplyRetention(Some("french".into()), 870)),
    );
    let text = std::fs::read_to_string(v.path.join("french/_stack.toml")).unwrap();
    assert!(text.contains("request_retention = 0.87"), "{text}");
    assert_eq!(
        model.state.notice.as_ref().unwrap().kind,
        NoticeKind::Info,
        "{:?}",
        model.state.notice
    );
}
