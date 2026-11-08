#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The quiet line when an answer takes far longer than the run's usual (STORY-0103).

mod common;

use common::views::{harness, open_model, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::{Latency, LatencyKind};
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_ui::app_settings_ui::SettingsTabMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

/// A run with item 1 revealed after `last_ms`, following five earlier reveals of 3 s.
fn revealed_after(last_ms: u32) -> studyfrog_ui::model::Model {
    let mut m = running_model(RunMode::Flip);
    m.state.now = 1.0;
    apply(&mut m, Msg::RunAction(Action::Flip));
    let run = m.state.rehearse.run.as_mut().unwrap();
    let flip = |item: u128, ms: u32| Latency {
        item: Uid(item),
        kind: LatencyKind::Flip,
        ms,
    };
    run.latencies = (0..5).map(|k| flip(50 + k, 3_000)).collect();
    run.latencies.push(flip(1, last_ms));
    m
}

const LINE: &str = "That one took 4.0 times as long as your usual answer in this run.";

#[test]
fn a_slow_reveal_gets_a_quiet_line_and_a_normal_one_does_not() {
    let mut h = harness(revealed_after(12_000));
    h.run_steps(3);
    assert!(h.query_by_label_contains(LINE).is_some());
    assert!(
        h.query_by_label("Paris").is_some(),
        "the answer is still shown"
    );
    let mut h = harness(revealed_after(4_000));
    h.run_steps(3);
    assert!(h.query_by_label_contains("times as long").is_none());
}

#[test]
fn the_line_can_be_switched_off_from_the_run_and_from_the_settings() {
    let mut h = harness(revealed_after(12_000));
    h.run_steps(3);
    h.get_by_label("Don't show this").click();
    h.run_steps(3);
    assert!(h.state().0.state.app.slow_answer_notice_off);
    assert!(h.query_by_label_contains(LINE).is_none());

    let mut m = open_model();
    let fx = apply(
        &mut m,
        Msg::SettingsTab(SettingsTabMsg::SlowAnswerNotice(false)),
    );
    assert!(matches!(&fx[..], [Effect::SaveAppSettings(s)] if s.slow_answer_notice_off));
    apply(
        &mut m,
        Msg::SettingsTab(SettingsTabMsg::SlowAnswerNotice(true)),
    );
    assert!(!m.state.app.slow_answer_notice_off);
}
