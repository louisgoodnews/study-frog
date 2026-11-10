#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for random typed answer sampling in setup and rehearsal.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::{SetupEdit, SetupForm, SetupStep, apply_edit, plan_from_form};
use studyfrog_ui::state::Screen;

#[test]
fn setup_form_typed_sample_validation_and_plan() {
    let mut form = SetupForm::default();
    form.mode = RunMode::Answer;
    form = apply_edit(form, SetupEdit::TypedSample(Some(50)));
    assert_eq!(form.typed_sample, Some(50));

    let plan = plan_from_form(&form, 42).unwrap();
    assert_eq!(plan.typed_sample, Some(50));

    // When mode is Flip, typed_sample is None in the plan
    let mut flip_form = form.clone();
    flip_form.mode = RunMode::Flip;
    let flip_plan = plan_from_form(&flip_form, 42).unwrap();
    assert_eq!(flip_plan.typed_sample, None);
}

#[test]
fn setup_screen_shows_typed_sampling_options_in_answer_mode() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    m.state.rehearse.step = SetupStep::How;
    apply(&mut m, Msg::Setup(SetupEdit::Mode(RunMode::Answer)));

    let mut h = harness(m);
    h.run_steps(4);

    assert!(h.query_by_label("Typed sampling:").is_some());
    assert!(h.query_by_label("All items (100%)").is_some());
    assert!(h.query_by_label("Random 50%").is_some());

    h.get_by_label("Random 50%").click();
    h.run_steps(2);

    assert_eq!(h.state().0.state.rehearse.form.typed_sample, Some(50));
}

#[test]
fn rehearsal_excludes_notes_from_typing_and_shows_flip_controls() {
    let mut plan = plan_spaced();
    plan.typed_sample = Some(100);
    let run = Run {
        uid: Uid(100),
        mode: RunMode::Answer,
        plan,
        queue: vec![Uid(1)],
        state: RunState::Idle,
        started_at: None,
    };
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    m.state.now = 0.0;
    apply(&mut m, Msg::RunReady(Ok(run)));
    let note_content = ItemContent {
        title: "Note Title".into(),
        prompt: "Note Prompt".into(),
        answer: "Note Answer".into(),
        previous: None,
        source: None,
        color: None,
        expected: None,
        occlusion: None,
        is_note: true,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(note_content)));

    let mut h = harness(m);
    h.run_steps(4);

    // Note should have "Show answer (Space)" button, NOT "Submit answer"
    assert!(h.query_by_label("Show answer (Space)").is_some());
    assert!(h.query_by_label("Submit answer").is_none());
}

#[test]
fn rehearsal_card_view_shows_typed_input_when_sampled() {
    let mut plan = plan_spaced();
    plan.typed_sample = Some(100); // 100% guarantees typing
    let run = Run {
        uid: Uid(100),
        mode: RunMode::Answer,
        plan,
        queue: vec![Uid(1)],
        state: RunState::Idle,
        started_at: None,
    };
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    m.state.now = 0.0;
    apply(&mut m, Msg::RunReady(Ok(run)));
    let card_content = ItemContent {
        title: "Card Title".into(),
        prompt: "Card Question".into(),
        answer: "Card Answer".into(),
        previous: None,
        source: None,
        color: None,
        expected: None,
        occlusion: None,
        is_note: false,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(card_content)));

    let mut h = harness(m);
    h.run_steps(4);

    // Card should have text input and "Submit answer" button
    assert!(h.query_by_label("Submit answer").is_some());
}
