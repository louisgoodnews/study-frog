//! The mock exam from a folder's notes or a lecture PDF (STORY-0186, STORY-0187, STORY-0207). Pure.

pub mod answer;
pub mod clock;
mod finish;
pub mod format_modal;
pub mod grading;
mod open;
mod save;
mod types;

pub use types::{ExamSource, MockExamEffect, MockExamMsg, MockExamUi, SlideChoice};

use crate::effect::Effect;
use crate::state::State;

/// Default question count for mock exam generation.
pub const EXAM_QUESTIONS: usize = 10;

/// Handle a mock-exam message.
pub fn update(state: State, msg: MockExamMsg) -> (State, Vec<Effect>) {
    match msg {
        MockExamMsg::Open => open::open(state),
        MockExamMsg::OpenSlides => open::open_slides(state),
        MockExamMsg::Slides(result) => (open::slides_listed(state, result), Vec::new()),
        MockExamMsg::FromSlide(path) => open::from_slide(state, path),
        MockExamMsg::FormatModal(m) => format_modal::update(state, m),
        MockExamMsg::Generated(result) => {
            let ui = map_ui(&state, |ui| match result {
                Ok(g) => {
                    let answers = answer::init_answers(&g.exam);
                    MockExamUi {
                        loading: false,
                        result: Some(g),
                        error: None,
                        slides: None,
                        answers,
                        ..ui
                    }
                }
                Err(e) => MockExamUi {
                    loading: false,
                    error: Some(e),
                    ..ui
                },
            });
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::Save => save::save(state),
        MockExamMsg::Saved(result) => save::saved(state, result),
        MockExamMsg::Start => {
            let now = state.now;
            let ui = map_ui(&state, |ui| MockExamUi {
                started: true,
                graded: None,
                started_at: Some(now),
                elapsed_sec: 0,
                locked: false,
                ..ui
            });
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::AnswerText(i, text) => {
            let ui = map_ui(&state, |ui| answer::set_text(ui, i, text));
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::AnswerGap(q_i, g_i, text) => {
            let ui = map_ui(&state, |ui| answer::set_gap(ui, q_i, g_i, text));
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::AnswerSingle(q_i, opt_i) => {
            let ui = map_ui(&state, |ui| answer::set_single(ui, q_i, opt_i));
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::AnswerMultiple(q_i, opt_i, on) => {
            let ui = map_ui(&state, |ui| answer::toggle_multiple(ui, q_i, opt_i, on));
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::Finish => finish::finish(state),
        MockExamMsg::Graded(graded) => {
            let ui = map_ui(&state, |ui| MockExamUi {
                judging: false,
                locked: false,
                started: false,
                graded: Some(graded),
                ..ui
            });
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::SetPassMark(pct) => {
            let ui = map_ui(&state, |ui| MockExamUi {
                pass_mark: Some(grading::clamp_pass_mark(pct)),
                ..ui
            });
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::ToggleAnswers => {
            let ui = map_ui(&state, |ui| MockExamUi {
                show_answers: !ui.show_answers,
                ..ui
            });
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::Close => (with_ui(state, None), Vec::new()),
    }
}

fn with_ui(state: State, ui: Option<MockExamUi>) -> State {
    State {
        rehearse: crate::setup::Rehearse {
            mock: ui,
            ..state.rehearse.clone()
        },
        ..state
    }
}

fn map_ui(state: &State, f: impl FnOnce(MockExamUi) -> MockExamUi) -> Option<MockExamUi> {
    state.rehearse.mock.clone().map(f)
}
