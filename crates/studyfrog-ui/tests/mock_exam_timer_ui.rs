#![allow(clippy::unwrap_used, clippy::expect_used)]
//! The mock exam's clock and locked answers (STORY-0185, BUG-0045), and PDF title (BUG-0039).

use studyfrog_rehearsal::exam_answer::ExamAnswer;
use studyfrog_service::exam_gen::generate;

use studyfrog_ui::effect::Effect;
use studyfrog_ui::modal_types::Modal;
use studyfrog_ui::setup::mock_exam::format_modal::FormatModalMsg;
use studyfrog_ui::setup::mock_exam::{MockExamEffect, MockExamMsg, update};
use studyfrog_ui::state::{Selection, State};

fn notes() -> &'static str {
    "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n"
}

fn confirm_open() -> State {
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, _) = update(state, MockExamMsg::Open);
    assert!(matches!(state.modal, Some(Modal::MockExamFormat(_))));
    let (state, _) = update(state, MockExamMsg::FormatModal(FormatModalMsg::Confirm));
    state
}

/// BUG-0045: the clock warns at ten and two minutes and submits the exam when its time is up.
#[test]
fn the_clock_counts_down_and_submits_the_exam_when_the_time_runs_out() {
    use studyfrog_ui::setup::mock_exam::clock::tick;
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let limit = g.exam.time_limit_min * 60;
    let state = confirm_open();
    let (mut state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    state.now = 100.0;
    let (state, _) = update(state, MockExamMsg::Start);
    assert_eq!(
        state.rehearse.mock.as_ref().unwrap().started_at,
        Some(100.0)
    );

    let mut state = state;
    state.now = 100.0 + f64::from(limit - 600);
    let (state, effects) = tick(state);
    let ui = state.rehearse.mock.clone().unwrap();
    assert_eq!(ui.elapsed_sec, limit - 600);
    assert!(effects.is_empty() && !ui.locked && ui.graded.is_none());

    let mut state = state;
    state.now = 100.0 + f64::from(limit) + 0.5;
    let (state, _) = tick(state);
    let ui = state.rehearse.mock.clone().unwrap();
    assert!(
        ui.graded.is_some(),
        "the time ran out, so the exam is graded: {ui:?}"
    );
    assert!(!ui.started);
}

/// BUG-0045: a submitted exam's answers cannot change until its result is shown.
#[test]
fn an_answer_is_locked_once_the_exam_is_submitted() {
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let state = confirm_open();
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, _) = update(state, MockExamMsg::Start);
    let (mut state, _) = update(state, MockExamMsg::AnswerText(0, "first".to_owned()));
    let mut ui = state.rehearse.mock.clone().unwrap();
    ui.locked = true;
    state.rehearse.mock = Some(ui);
    let (state, _) = update(state, MockExamMsg::AnswerText(0, "changed".to_owned()));
    assert_eq!(
        state.rehearse.mock.unwrap().answers[0],
        ExamAnswer::Text("first".to_owned())
    );
}

/// BUG-0039: an exam made from a PDF is titled with the PDF's file name.
#[test]
fn an_exam_from_a_pdf_is_titled_with_its_file_name() {
    use studyfrog_ui::setup::mock_exam::{MockExamUi, SlideChoice};
    let mut state = State::default();
    state.rehearse.mock = Some(MockExamUi {
        slides: Some(vec![SlideChoice {
            path: "Vorlesung2.pdf".to_owned(),
            title: "untitled".to_owned(),
        }]),
        ..MockExamUi::default()
    });
    let (state, _) = update(state, MockExamMsg::FromSlide("Vorlesung2.pdf".to_owned()));
    assert!(matches!(state.modal, Some(Modal::MockExamFormat(_))));
    let (state, effects) = update(state, MockExamMsg::FormatModal(FormatModalMsg::Confirm));
    assert_eq!(
        state.rehearse.mock.unwrap().title,
        "Mock exam: Vorlesung2.pdf"
    );
    assert!(
        matches!(effects.as_slice(), [Effect::MockExam(MockExamEffect::Generate { title, .. })] if title == "Mock exam: Vorlesung2.pdf"),
        "{effects:?}"
    );
}
