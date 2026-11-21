#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The case-study rehearsal (STORY-0181): opening a stack's cases, checking an answer against the
//! rubric, moving on, and what the view shows.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::case_study::CaseStudyItem;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::setup::case::{CaseMsg, CaseUi, update};
use studyfrog_ui::state::{Selection, State};

fn case(choice: &str) -> CaseStudyItem {
    CaseStudyItem {
        vignette: "A clinic wants a short screen for anxiety. Staff are rushed and the test must be quick to score.".to_owned(),
        decision_prompt: "Which instrument should the clinic use?".to_owned(),
        options: vec!["Short screen".to_owned(), "Full interview".to_owned()],
        expected_choices: vec![choice.to_owned()],
        rubric_criteria: vec!["addresses scoring speed".to_owned(), "cites criterion validity".to_owned()],
    }
}

fn loaded(cases: Vec<CaseStudyItem>) -> CaseUi {
    CaseUi {
        cases: cases
            .into_iter()
            .enumerate()
            .map(|(i, c)| (Uid(i as u128 + 1), c))
            .collect(),
        picked: vec![false, false],
        ..CaseUi::default()
    }
}

fn with_case(cs: CaseUi) -> State {
    let mut state = State::default();
    state.rehearse.case_study = Some(cs);
    state
}

fn send(state: State, m: CaseMsg) -> State {
    update(state, m).0
}

fn view(state: &State) -> &CaseUi {
    state.rehearse.case_study.as_ref().expect("the case view")
}

#[test]
fn opening_the_cases_of_a_stack_reads_them() {
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, effects) = update(state, CaseMsg::Open);
    assert!(view(&state).loading);
    assert_eq!(effects, vec![Effect::LoadCases(Some("bio".to_owned()))]);
}

#[test]
fn a_stack_without_cases_says_so() {
    let state = send(
        with_case(CaseUi::default()),
        CaseMsg::Loaded(Ok(Vec::new())),
    );
    assert!(
        view(&state)
            .error
            .as_deref()
            .is_some_and(|e| e.contains("no case studies"))
    );
}

#[test]
fn the_right_choice_and_a_full_justification_are_solid() {
    let state = with_case(loaded(vec![case("Short screen")]));
    let state = send(state, CaseMsg::Pick(0, true));
    let state = send(state, CaseMsg::Rationale("A short screen keeps the scoring speed the staff need, and its criterion validity is documented for this population.".to_owned()));
    let state = send(state, CaseMsg::Submit);
    let report = view(&state).report.clone().expect("a report");
    assert!(report.choice_valid);
    assert!(report.criteria.iter().all(|c| c.met), "{report:?}");
}

#[test]
fn a_wrong_choice_is_reported_and_the_justification_is_scored_on_its_own() {
    let state = with_case(loaded(vec![case("Short screen")]));
    let state = send(state, CaseMsg::Pick(1, true));
    let state = send(state, CaseMsg::Submit);
    let report = view(&state).report.clone().expect("a report");
    assert!(!report.choice_valid);
    assert!(report.criteria.iter().all(|c| !c.met));
}

#[test]
fn next_wraps_to_the_first_case_and_clears_the_answer() {
    let state = with_case(loaded(vec![case("Short screen"), case("Full interview")]));
    let state = send(state, CaseMsg::Pick(0, true));
    let state = send(state, CaseMsg::Submit);
    let state = send(state, CaseMsg::Next);
    assert_eq!(view(&state).current, 1);
    assert!(view(&state).report.is_none());
    assert_eq!(view(&state).picked, vec![false, false]);
    let state = send(state, CaseMsg::Next);
    assert_eq!(view(&state).current, 0);
}

#[test]
fn the_view_shows_the_case_and_the_evaluation() {
    let state = with_case(loaded(vec![case("Short screen")]));
    let state = send(state, CaseMsg::Pick(0, true));
    let state = send(state, CaseMsg::Submit);
    let mut h = harness(open_model_with(state));
    h.run_steps(4);
    for text in [
        "Case 1 of 1",
        "Which instrument should the clinic use?",
        "Check my answer",
        "Choice: the expected one",
    ] {
        assert!(h.query_by_label_contains(text).is_some(), "missing {text}");
    }
    assert!(h.query_all_by_label_contains("Missing").count() >= 1);
}

fn open_model_with(state: State) -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.screen = studyfrog_ui::state::Screen::Rehearse;
    m.state.rehearse = state.rehearse;
    m
}
