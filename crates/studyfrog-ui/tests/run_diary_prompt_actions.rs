#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for TASK-0173: Interaction actions on the post-rehearsal diary prompt.

mod common;

use common::views::{harness, running_model};
use eframe::egui::Key;
use egui_kittest::kittest::Queryable;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_ui::diary_types::DiaryEffect;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::state::Screen;

fn finish_run(h: &mut egui_kittest::Harness<'_, common::views::App>) {
    h.run_steps(2);
    h.key_press(Key::Space);
    h.run_steps(2);
    h.key_press(Key::Num3);
    h.run_steps(2);
    h.key_press(Key::S);
    h.run_steps(2);
    let state = h
        .state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .run
        .state;
    assert_eq!(state, RunState::Finished);
}

#[test]
fn skipping_dismisses_prompt_cleanly_and_allows_leaving_immediately() {
    let mut h = harness(running_model(RunMode::Flip));
    finish_run(&mut h);

    assert!(
        h.query_by_label_contains("Log your session in the diary?")
            .is_some()
    );

    h.get_by_label("Skip (Esc)").click();
    h.run_steps(2);

    assert!(
        h.query_by_label_contains("Log your session in the diary?")
            .is_none(),
        "prompt is dismissed cleanly"
    );
    assert!(
        !h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::Diary(DiaryEffect::SaveEntry(_)))),
        "nothing is saved when skipped"
    );

    h.get_by_label("Back to setup").click();
    h.run_steps(2);
    assert!(h.state().0.state.rehearse.run.is_none());
}

#[test]
fn escape_skips_prompt_and_second_escape_leaves_summary() {
    let mut h = harness(running_model(RunMode::Flip));
    finish_run(&mut h);

    assert!(
        h.query_by_label_contains("Log your session in the diary?")
            .is_some()
    );

    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Log your session in the diary?")
            .is_none(),
        "escape dismissed prompt"
    );

    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        h.state().0.state.rehearse.run.is_none(),
        "second escape left the summary"
    );
}

#[test]
fn clicking_back_to_setup_leaves_immediately_without_blocking() {
    let mut h = harness(running_model(RunMode::Flip));
    finish_run(&mut h);

    h.get_by_label("Rate Energy level level 3").click();
    h.run_steps(2);

    h.get_by_label("Back to setup").click();
    h.run_steps(2);
    assert!(
        h.state().0.state.rehearse.run.is_none(),
        "left summary immediately without blocking"
    );
}

#[test]
fn more_details_expands_to_full_diary_editor() {
    let mut h = harness(running_model(RunMode::Flip));
    finish_run(&mut h);

    let run_uid = h
        .state()
        .0
        .state
        .rehearse
        .run
        .as_ref()
        .unwrap()
        .session
        .run
        .uid;
    h.get_by_label("Rate Energy level level 5").click();
    h.run_steps(2);

    h.get_by_label("More details...").click();
    h.run_steps(2);

    assert_eq!(h.state().0.state.screen, Screen::Diary);
    let active = h
        .state()
        .0
        .state
        .diary
        .active_entry
        .as_ref()
        .expect("draft exists");
    assert_eq!(active.runs, vec![run_uid]);
    assert_eq!(active.ratings.get("energy"), Some(&RatingValue::Int(5)));
}
