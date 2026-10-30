#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The rehearsal screen's real widgets, headless: clicks, keys, pause, summary, accessibility.

mod common;

use common::views::{assert_all_named, harness, open_model, running_model};
use eframe::egui::Key;
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::EventKind;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::SetupStep;
use studyfrog_ui::state::Screen;

fn session_state(h: &egui_kittest::Harness<'_, common::views::App>) -> RunState {
    h.state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .run
        .state
}

fn rated(h: &egui_kittest::Harness<'_, common::views::App>) -> Vec<Rating> {
    let run = h.state().0.state.rehearse.run.as_ref().unwrap();
    run.events
        .iter()
        .filter_map(|e| {
            if let EventKind::ItemRated(r) = e.kind {
                Some(r)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn the_prompt_is_shown_then_revealed_then_rated_with_the_mouse() {
    let mut h = harness(running_model(RunMode::Flip));
    h.run_steps(2);
    assert!(
        h.query_by_label("Capital of France?").is_some() && h.query_by_label("1 / 2").is_some()
    );
    assert!(
        h.query_by_label("Paris").is_none(),
        "the answer is hidden until it is revealed"
    );
    assert!(
        h.query_by_label("Good (3)").is_none(),
        "no rating before the reveal"
    );

    h.get_by_label("Show answer (Space)").click();
    h.run_steps(2);
    assert!(h.query_by_label("Paris").is_some());
    h.get_by_label("Good (3)").click();
    h.run_steps(2);
    assert_eq!(rated(&h), [Rating::Good]);
    assert!(
        h.query_by_label("7 x 8?").is_some() && h.query_by_label("2 / 2").is_some(),
        "on to the next item"
    );
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::RecordStep { .. })),
        "every step is saved"
    );
}

#[test]
fn the_whole_run_can_be_done_from_the_keyboard() {
    let mut h = harness(running_model(RunMode::Flip));
    h.run_steps(2);
    h.key_press(Key::Space);
    h.run_steps(2);
    h.key_press(Key::Num4);
    h.run_steps(2);
    h.key_press(Key::S); // skip the second item
    h.run_steps(2);
    assert_eq!(rated(&h), [Rating::Easy]);
    assert_eq!(session_state(&h), RunState::Finished);
    assert!(h.query_by_label("Run finished").is_some());
    assert!(
        h.query_by_label("Easy: 1").is_some()
            && h.query_by_label("rated").is_some()
            && h.query_by_label("skipped").is_some()
    );
}

#[test]
fn escape_stops_the_run_and_the_summary_offers_the_way_back() {
    let mut h = harness(running_model(RunMode::Flip));
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert_eq!(session_state(&h), RunState::Aborted);
    assert!(
        h.query_by_label("Run stopped").is_some()
            && h.query_by_label_contains("nothing to rehearse").is_some()
    );
    h.get_by_label("Back to setup").click();
    h.run_steps(2);
    assert!(h.state().0.state.rehearse.run.is_none());
    assert!(
        h.query_by_label("Next: configure").is_some(),
        "back at the first step"
    );
}

#[test]
fn pausing_hides_the_question_and_resuming_brings_it_back() {
    let mut h = harness(running_model(RunMode::Flip));
    h.run_steps(2);
    h.key_press(Key::P);
    h.run_steps(2);
    assert_eq!(session_state(&h), RunState::Paused);
    assert!(
        h.query_by_label("Paused").is_some() && h.query_by_label("Capital of France?").is_none()
    );
    h.get_by_label("Resume (P)").click();
    h.run_steps(2);
    assert_eq!(session_state(&h), RunState::Running);
    assert!(h.query_by_label("Capital of France?").is_some());
}

#[test]
fn typing_an_answer_does_not_trigger_shortcuts_and_submitting_reveals_the_reference() {
    let mut h = harness(running_model(RunMode::Answer));
    h.run_steps(2);
    h.get_by_label("Your answer").focus();
    h.get_by_label("Your answer").type_text("paris s 1 p");
    h.run_steps(2);
    assert_eq!(
        session_state(&h),
        RunState::Running,
        "space, S, 1 and P were typed, not interpreted"
    );
    assert_eq!(
        h.state().0.state.rehearse.run.as_ref().unwrap().answer,
        "paris s 1 p"
    );
    assert!(h.query_by_label("Paris").is_none());

    h.get_by_label("Submit answer").click();
    h.run_steps(2);
    let run = h.state().0.state.rehearse.run.as_ref().unwrap();
    assert!(
        run.events
            .iter()
            .any(|e| e.kind == EventKind::AnswerSubmitted("paris s 1 p".into()))
    );
    assert!(
        h.query_by_label("Paris").is_some(),
        "the reference answer is shown for comparison"
    );
}

#[test]
fn the_start_button_needs_items_and_asks_for_a_run() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    apply(&mut m, Msg::PreviewReady(Ok(0)));
    apply(&mut m, Msg::SetupStep(SetupStep::Review));
    let mut h = harness(m);
    h.run();
    h.get_by_label("Start rehearsal").click();
    h.run();
    assert!(
        !h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::StartRun { .. })),
        "nothing to rehearse, nothing starts"
    );

    apply(&mut h.state_mut().0, Msg::PreviewReady(Ok(5)));
    h.run();
    h.get_by_label("Start rehearsal").click();
    h.run_steps(3);
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::StartRun { .. }))
    );
}

#[test]
fn every_control_of_every_rehearsal_state_has_an_accessible_name() {
    for mode in [RunMode::Flip, RunMode::Answer] {
        let mut h = harness(running_model(mode));
        h.run_steps(2);
        assert_all_named(&h, "prompt shown");
        h.key_press(Key::Space);
        h.run_steps(2);
        assert_all_named(&h, "answer revealed");
        h.key_press(Key::P);
        h.run_steps(2);
        assert_all_named(&h, "paused");
        h.key_press(Key::Escape);
        h.run_steps(2);
        assert_all_named(&h, "summary");
    }
}
