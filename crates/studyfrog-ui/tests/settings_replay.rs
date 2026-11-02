#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Settings shows whether familiarity helps on the user's own history, and never switches it off itself.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_report::replay::{ReplayReport, Verdict, replay};
use studyfrog_srs::params::params_default;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn report(verdict: Verdict) -> ReplayReport {
    ReplayReport {
        verdict,
        scored: 1500,
        loss: (0.41, 0.43),
        workload_percent: -12.0,
        ..replay(&[], &params_default())
    }
}

#[test]
fn opening_settings_asks_for_the_reports_and_the_verdict_is_told_in_words() {
    let (_v, mut model, mut worker) = opened("settings-replay");
    let fx = apply(&mut model, Msg::Navigate(Screen::Settings));
    assert!(fx.contains(&Effect::BuildReports));
    send(&mut model, &mut worker, Msg::Navigate(Screen::Settings));
    let r = &model.state.reports.as_ref().unwrap().replay;
    assert_eq!(
        r.verdict,
        Verdict::TooFewReviews,
        "a new vault has no history"
    );

    let mut h = harness(model);
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Too few reviews for a verdict")
            .is_some()
    );

    h.state_mut().0.state.reports.as_mut().unwrap().replay = report(Verdict::PlainBetter);
    let before = h.state().0.state.app.clone();
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("plain FSRS predicts your recall better")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Nothing is changed for you")
            .is_some()
    );
    assert!(h.query_by_label_contains("12 % less").is_some());
    assert!(
        h.query_by_label_contains("plain 0.410, with familiarity 0.430")
            .is_some()
    );
    assert_all_named(&h, "settings with the verdict");
    assert_eq!(
        h.state().0.state.app,
        before,
        "the verdict never changes a setting"
    );
}
