#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The case-study rehearsal (STORY-0181): opening a stack's cases, checking an answer against the
//! rubric, moving on, and what the view shows.
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::case_study::CaseStudyItem;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectBrowse;
use studyfrog_ui::setup::case::{CaseMsg, CaseUi, update};
use studyfrog_ui::state::{Selection, State};

mod common;

fn send(state: State, m: CaseMsg) -> State {
    update(state, m).0
}

fn view(state: &State) -> &CaseUi {
    state.rehearse.case_study.as_ref().expect("the case view")
}

fn with_provider(url: &str) -> State {
    let mut state = with_case(loaded(vec![case("Short screen")]));
    state.agent.connections.providers = vec![provider(url)];
    state.agent.selected = Some("p".to_owned());
    state
}

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

fn provider(url: &str) -> studyfrog_agent::provider::Provider {
    studyfrog_agent::provider::Provider {
        name: "p".to_owned(),
        protocol: studyfrog_agent::provider::Protocol::Ollama,
        base_url: url.to_owned(),
        model: "m".to_owned(),
        allowed: false,
    }
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
    assert_eq!(
        effects,
        vec![Effect::Browse(EffectBrowse::LoadCases(Some(
            "diag".to_owned()
        )))]
    );
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
