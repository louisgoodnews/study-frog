#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The behavioural analytics on the reports screen (EPIC-034).

mod common;

use common::reports::{analytics, bundle};
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_report::retention_survival::rate_of;
use studyfrog_ui::explorer_ui::ReportTab;
use studyfrog_ui::state::Screen;

fn model_on(tab: ReportTab) -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = tab;
    let mut b = bundle();
    b.analytics = analytics();
    m.state.reports = Some(b);
    m
}

#[test]
fn the_memory_tab_shows_true_retention_and_the_curves_by_stability_as_text() {
    let m = model_on(ReportTab::Memory);
    let s = m.state.reports.as_ref().unwrap().analytics.survival.clone();
    let mut h = harness(m);
    h.run_steps(4);
    let t = rate_of(&s.true_retention).unwrap();
    let want = format!(
        "True retention: {:.0} % of {}",
        t * 100.0,
        s.true_retention.reviews
    );
    assert!(h.query_by_label_contains(&want).is_some(), "missing {want}");
    for text in [
        "Young items (stability under 21 days)",
        "Overdue (resilience)",
        "Your memory half-life for a typical item",
        "Forgetting curves by stability",
        "Stability 0-7 days",
        "Stability 90+ days",
    ] {
        assert!(h.query_by_label_contains(text).is_some(), "missing {text}");
    }
    for text in ["measured,", "predicted ("] {
        assert!(
            h.query_all_by_label_contains(text).count() >= 2,
            "missing {text}"
        );
    }
    assert_all_named(&h, "survival report");
}

#[test]
fn every_tab_has_a_word_for_the_screenshot_option() {
    use studyfrog_ui::explorer_ui::{TAB_WORDS, tab_named};
    for (word, tab) in TAB_WORDS {
        assert_eq!(tab_named(word), Some(tab));
    }
    assert_eq!(tab_named("memory"), Some(ReportTab::Memory));
    assert_eq!(tab_named("nothing"), None);
}
