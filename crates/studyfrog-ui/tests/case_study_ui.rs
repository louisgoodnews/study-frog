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

fn provider(url: &str) -> studyfrog_agent::provider::Provider {
    studyfrog_agent::provider::Provider {
        name: "p".to_owned(),
        protocol: studyfrog_agent::provider::Protocol::Ollama,
        base_url: url.to_owned(),
        model: "m".to_owned(),
        allowed: false,
    }
}

fn with_provider(url: &str) -> State {
    let mut state = with_case(loaded(vec![case("Short screen")]));
    state.agent.connections.providers = vec![provider(url)];
    state.agent.selected = Some("p".to_owned());
    state
}

#[test]
fn a_local_model_is_asked_to_check_the_justification() {
    let state = send(
        with_provider("http://localhost:11434"),
        CaseMsg::Pick(0, true),
    );
    let (state, effects) = update(state, CaseMsg::Submit);
    assert!(
        view(&state).report.as_ref().is_some_and(|r| !r.from_model),
        "the keyword check shows at once"
    );
    assert!(view(&state).pending);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::AgentChat {
                purpose: studyfrog_ui::agent_types::ChatPurpose::Case,
                ..
            }]
        ),
        "{effects:?}"
    );
    let reply = r#"{"choice_valid": true, "criteria_results": [{"criterion_text": "addresses scoring speed", "met": true, "feedback": "Quoted."}, {"criterion_text": "cites criterion validity", "met": false, "feedback": "Missing."}], "caveats": "Check the norms."}"#;
    let state = send(state, CaseMsg::ModelReplied(Ok(reply.to_owned())));
    assert!(view(&state).checked_by_model);
    assert!(
        view(&state)
            .report
            .as_ref()
            .is_some_and(|r| r.from_model && r.choice_valid)
    );
    assert!(!view(&state).pending);
}

#[test]
fn an_unreadable_model_reply_keeps_the_keyword_check_and_says_why() {
    let state = send(
        with_provider("http://localhost:11434"),
        CaseMsg::Pick(0, true),
    );
    let (state, _) = update(state, CaseMsg::Submit);
    let state = send(state, CaseMsg::ModelReplied(Ok("I cannot say.".to_owned())));
    assert!(!view(&state).checked_by_model);
    assert!(
        view(&state)
            .note
            .as_deref()
            .is_some_and(|n| n.contains("could not check"))
    );
    assert!(view(&state).report.as_ref().is_some_and(|r| !r.from_model));
}

#[test]
fn a_remote_model_is_not_sent_the_students_text() {
    let state = send(
        with_provider("https://api.example.com"),
        CaseMsg::Pick(0, true),
    );
    let (state, effects) = update(state, CaseMsg::Submit);
    assert!(effects.is_empty(), "{effects:?}");
    assert!(
        view(&state)
            .note
            .as_deref()
            .is_some_and(|n| n.contains("Only a local model"))
    );
}

#[test]
fn the_case_study_reads_the_stack_ticked_in_the_setup_not_the_open_one() {
    let mut state = State {
        vault: Some(common::views::vault_with_stacks(&[(1, "bio"), (2, "diag")])),
        ..State::default()
    };
    state.rehearse.form.stacks = vec![Uid(2)];
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (_, effects) = update(state, CaseMsg::Open);
    assert_eq!(effects, vec![Effect::LoadCases(Some("diag".to_owned()))]);
}

#[test]
fn with_nothing_ticked_or_open_the_case_study_says_to_tick_or_open_one() {
    let (state, effects) = update(State::default(), CaseMsg::Open);
    assert!(effects.is_empty());
    let error = view(&state).error.clone().unwrap_or_default();
    assert!(
        error.contains("Tick a stack") && error.contains("open one"),
        "{error}"
    );
}
