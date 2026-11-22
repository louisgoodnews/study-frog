#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The mock exam from notes (STORY-0186): asking for it, showing it, and saving it.

use studyfrog_service::exam_gen::generate;

use studyfrog_ui::effect::Effect;
use studyfrog_ui::setup::mock_exam::{EXAM_QUESTIONS, MockExamEffect, MockExamMsg, update};
use studyfrog_ui::state::{Selection, State};

fn notes() -> &'static str {
    "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n"
}

#[test]
fn opening_a_stack_asks_for_an_exam_of_its_notes() {
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, effects) = update(state, MockExamMsg::Open);
    assert!(state.rehearse.mock.as_ref().is_some_and(|m| m.loading));
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Generate { folder: Some(f), total, provider: None, .. })]
                if f == "bio" && *total == EXAM_QUESTIONS
        ),
        "{effects:?}"
    );
}

#[test]
fn opening_with_nothing_chosen_says_so() {
    let (state, effects) = update(State::default(), MockExamMsg::Open);
    assert!(effects.is_empty());
    assert!(
        state
            .rehearse
            .mock
            .as_ref()
            .and_then(|m| m.error.clone())
            .is_some()
    );
}

#[test]
fn a_made_exam_is_shown_and_its_answers_toggle() {
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let state = State::default();
    let (state, _) = update(state, MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let ui = state.rehearse.mock.clone().unwrap();
    assert!(!ui.loading && ui.result.is_some());
    let (state, _) = update(state, MockExamMsg::ToggleAnswers);
    assert!(state.rehearse.mock.as_ref().unwrap().show_answers);
}

#[test]
fn saving_asks_for_the_note_and_the_outcome_is_reported() {
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, _) = update(state, MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, effects) = update(state, MockExamMsg::Save);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Save { .. })]
        ),
        "{effects:?}"
    );
    let (state, _) = update(state, MockExamMsg::Saved(Ok("bio/klausur.md".to_owned())));
    assert!(
        state
            .notice
            .as_ref()
            .is_some_and(|n| n.text.contains("bio/klausur.md"))
    );
    let (state, _) = update(state, MockExamMsg::Saved(Err("disk full".to_owned())));
    assert!(
        state
            .notice
            .as_ref()
            .is_some_and(|n| n.text.contains("disk full"))
    );
}
