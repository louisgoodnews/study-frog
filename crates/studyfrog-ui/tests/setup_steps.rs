#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The run setup as three steps: what, how, review.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::{Order, ScheduleEffect};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::{Preset, SetupEdit, SetupStep};
use studyfrog_ui::state::Screen;

fn setup_model() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    apply(&mut m, Msg::PreviewReady(Ok(7)));
    m
}

#[test]
fn the_setup_starts_on_what_and_the_stacks_are_ticked_there() {
    let mut h = harness(setup_model());
    h.run();
    assert_eq!(h.state().0.state.rehearse.step, SetupStep::What);
    for text in [
        "1  What",
        "2  How",
        "3  Review",
        "Step 1 of 3",
        "Next: configure",
        "Kinds of item",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert!(
        h.query_by_label("Start rehearsal").is_none(),
        "starting belongs to the last step"
    );
    assert!(h.query_by_label("Spaced repetition").is_none());
    assert_all_named(&h, "setup what");
}

#[test]
fn next_and_back_walk_the_steps_and_the_step_bar_jumps() {
    let mut h = harness(setup_model());
    h.run();
    h.get_by_label("Next: configure").click();
    h.run();
    assert!(
        h.query_by_label("Spaced repetition").is_some()
            && h.query_by_label("Step 2 of 3").is_some()
    );
    assert_all_named(&h, "setup how");
    h.get_by_label("Next: review").click();
    h.run();
    assert!(
        h.query_by_label("Start rehearsal").is_some() && h.query_by_label("Change What").is_some()
    );
    assert_all_named(&h, "setup review");
    h.get_by_label("Change What").click();
    h.run();
    assert_eq!(h.state().0.state.rehearse.step, SetupStep::What);
    h.get_by_label("3  Review").click();
    h.run();
    h.get_by_label("Back").click();
    h.run();
    assert_eq!(h.state().0.state.rehearse.step, SetupStep::How);
}

#[test]
fn a_preset_keeps_what_was_chosen_in_the_first_step() {
    let mut m = setup_model();
    let uid = m.state.vault.as_ref().unwrap().overview.stacks[0].uid;
    apply(&mut m, Msg::Setup(SetupEdit::Stacks(vec![uid], true)));
    apply(&mut m, Msg::Setup(SetupEdit::Tags("verbs".into())));
    apply(&mut m, Msg::Setup(SetupEdit::Preset(Preset::Hardest)));
    let f = &m.state.rehearse.form;
    assert_eq!((f.stacks.clone(), f.tags.as_str()), (vec![uid], "verbs"));
}

#[test]
fn the_review_names_the_chosen_stack_and_the_plain_defaults() {
    let mut m = setup_model();
    let (uid, title) = {
        let s = &m.state.vault.as_ref().unwrap().overview.stacks[0];
        (s.uid, s.title.clone())
    };
    apply(&mut m, Msg::Setup(SetupEdit::Stacks(vec![uid], true)));
    apply(&mut m, Msg::SetupStep(SetupStep::Review));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label_contains(&title).is_some());
    assert!(h.query_by_label_contains("flip cards").is_some());
    assert!(
        h.query_by_label_contains("update the review schedule")
            .is_some()
    );
}

#[test]
fn shuffling_a_spaced_run_keeps_it_updating_the_schedule() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    let fx = apply(&mut m, Msg::Setup(SetupEdit::Shuffle(true)));
    let [Effect::PreviewQueue { plan, .. }] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(
        (plan.order, plan.shuffle, plan.effect),
        (Order::Due, true, ScheduleEffect::UpdateSchedule)
    );
    apply(&mut m, Msg::Setup(SetupEdit::Preset(Preset::Hardest)));
    assert!(
        !m.state.rehearse.form.shuffle,
        "a preset starts from its own plan"
    );
}
