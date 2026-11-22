#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The mock exam from a lecture PDF and the model (STORY-0186, STORY-0187): which model writes or grades it,
//! what the student is told when there is none, and the PDF listing.

use studyfrog_service::exam_gen::generate;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::setup::mock_exam::{
    ExamSource, MockExamEffect, MockExamMsg, SlideChoice, update,
};
use studyfrog_ui::state::{Selection, State};

fn notes() -> &'static str {
    "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n"
}

fn provider(url: &str, allowed: bool) -> studyfrog_agent::provider::Provider {
    studyfrog_agent::provider::Provider {
        name: "p".to_owned(),
        protocol: studyfrog_agent::provider::Protocol::Ollama,
        base_url: url.to_owned(),
        model: "m".to_owned(),
        allowed,
    }
}

fn with_selected(url: &str, allowed: bool) -> State {
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    state.agent.connections.providers = vec![provider(url, allowed)];
    state.agent.selected = Some("p".to_owned());
    state
}

#[test]
fn with_no_model_selected_the_exam_says_it_is_made_from_templates() {
    let (state, effects) = update(
        {
            let mut s = State::default();
            s.browser.selection = Selection::Stack("bio".to_owned());
            s
        },
        MockExamMsg::Open,
    );
    let note = state.rehearse.mock.as_ref().and_then(|m| m.note.clone());
    assert!(note.is_some_and(|n| n.contains("No model is selected")));
    assert!(matches!(
        effects.as_slice(),
        [Effect::MockExam(MockExamEffect::Generate {
            provider: None,
            ..
        })]
    ));
}

#[test]
fn a_remote_model_that_may_not_see_the_notes_is_not_sent_them() {
    let (state, effects) = update(
        with_selected("https://api.example.com/v1", false),
        MockExamMsg::Open,
    );
    assert!(
        state
            .rehearse
            .mock
            .as_ref()
            .and_then(|m| m.note.clone())
            .is_some_and(|n| n.contains("remote"))
    );
    assert!(matches!(
        effects.as_slice(),
        [Effect::MockExam(MockExamEffect::Generate {
            provider: None,
            ..
        })]
    ));
}

#[test]
fn a_local_model_writes_the_exam_and_no_note_is_needed() {
    let (state, effects) = update(
        with_selected("http://localhost:11434", false),
        MockExamMsg::Open,
    );
    assert!(state.rehearse.mock.as_ref().unwrap().note.is_none());
    assert!(matches!(
        effects.as_slice(),
        [Effect::MockExam(MockExamEffect::Generate {
            provider: Some(_),
            ..
        })]
    ));
}

#[test]
fn a_lecture_pdf_is_listed_then_made_into_an_exam() {
    let (state, effects) = update(State::default(), MockExamMsg::OpenSlides);
    assert!(matches!(
        effects.as_slice(),
        [Effect::MockExam(MockExamEffect::ListSlides)]
    ));
    let list = vec![SlideChoice {
        path: "folien/norm.pdf".to_owned(),
        title: "Normierung".to_owned(),
    }];
    let (state, _) = update(state, MockExamMsg::Slides(Ok(list.clone())));
    assert_eq!(state.rehearse.mock.as_ref().unwrap().slides, Some(list));
    let (state, effects) = update(state, MockExamMsg::FromSlide("folien/norm.pdf".to_owned()));
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Generate { source: ExamSource::Slide(p), title, .. })]
                if p == "folien/norm.pdf" && title == "Mock exam: Normierung"
        ),
        "{effects:?}"
    );
    assert!(state.rehearse.mock.as_ref().unwrap().loading);
}

#[test]
fn a_failed_pdf_listing_says_why() {
    let (state, _) = update(State::default(), MockExamMsg::OpenSlides);
    let (state, _) = update(
        state,
        MockExamMsg::Slides(Err("no vault is open".to_owned())),
    );
    let ui = state.rehearse.mock.unwrap();
    assert!(!ui.loading && ui.error.as_deref() == Some("no vault is open"));
}

#[test]
fn finishing_with_no_model_grades_by_the_keyword_check_at_once() {
    use studyfrog_ui::setup::mock_exam::grading::total;
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let (state, _) = update(State::default(), MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, effects) = update(state, MockExamMsg::Finish);
    assert!(effects.is_empty());
    let ui = state.rehearse.mock.unwrap();
    let graded = ui.graded.expect("graded at once");
    assert!(graded.iter().all(|q| !q.from_model));
    assert_eq!(total(&graded).earned, 0);
}

#[test]
fn finishing_with_a_local_model_asks_it_to_grade_and_shows_its_result() {
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let n = g.keys.len();
    let (state, _) = update(
        with_selected("http://localhost:11434", false),
        MockExamMsg::Open,
    );
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, effects) = update(state, MockExamMsg::Finish);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Grade { answers, .. })] if answers.len() == n
        ),
        "{effects:?}"
    );
    assert!(state.rehearse.mock.as_ref().unwrap().judging);
    let (state, _) = update(state, MockExamMsg::Graded(Vec::new()));
    let ui = state.rehearse.mock.unwrap();
    assert!(!ui.judging && ui.graded.is_some());
}
