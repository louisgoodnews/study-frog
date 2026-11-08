#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The behavioural analytics on the reports screen (EPIC-034).

mod common;

use common::analytics::analytics;
use common::reports::bundle;
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
    b.analytics = Box::new(analytics());
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

#[test]
fn the_habits_tab_names_the_peak_the_trough_and_the_fatigue_point() {
    let mut h = harness(model_on(ReportTab::Habits));
    h.run_steps(4);
    for text in [
        "Your recall is highest between 09:00 and 11:00 (95 % of 40, median 3.0 s).",
        "It is lowest between 20:00 and 22:00 (50 % of 40, median 6.0 s).",
        "09:00",
        "21:00",
        "Best on Monday, weakest on Monday.",
        "Your accuracy holds for about 15 minutes (about 15 items); after that the median time to answer rises 67 % and recall drops 30 points. Runs of about 15 minutes suit you.",
        "First ten minutes: 90 % of 30, median 3.0 s",
        "15-20 min",
        "20-30 min",
        "items 1-10",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_eq!(
        h.query_all_by_label("of 20, median 3.0 s, 90 % under 3.0 s")
            .count(),
        2,
        "09:00 and 10:00"
    );
    assert_all_named(&h, "habits report");
}

#[test]
fn a_higher_threshold_withholds_the_recommendation() {
    let mut m = model_on(ReportTab::Habits);
    studyfrog_ui::model::apply(
        &mut m,
        studyfrog_ui::msg::Msg::Explorer(studyfrog_ui::explorer_ui::ExplorerMsg::MinBucket(50)),
    );
    assert_eq!(m.state.explorer.min_bucket, 50);
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Your recall is highest")
            .is_none()
    );
    assert!(
        h.query_by_label_contains("two hours in a row need 50 reviews each")
            .is_some()
    );
    h.get_by_label("10").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.explorer.min_bucket, 10);
    assert!(
        h.query_by_label_contains("Your recall is highest")
            .is_some()
    );
}
