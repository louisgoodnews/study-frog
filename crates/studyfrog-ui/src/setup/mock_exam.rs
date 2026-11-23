//! The mock exam from a folder's notes or a lecture PDF (STORY-0186), and its grading (STORY-0187): asking
//! for it, showing what was made, grading the answers, and saving it as a note. The generation is in
//! `studyfrog-service::exam_gen`. Pure: no egui, no I/O.

use studyfrog_agent::provider::Provider;
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::mock_exam::MockExam;
use studyfrog_service::exam_gen::Generated;

pub mod clock;
mod finish;
pub mod grading;
mod open;
mod save;

use crate::effect::Effect;
use crate::state::State;

/// Questions asked for an exam from the notes.
pub const EXAM_QUESTIONS: usize = 10;

/// Messages of the mock exam.
#[derive(Debug, Clone, PartialEq)]
pub enum MockExamMsg {
    /// Make an exam from the stack that is open.
    Open,
    /// List the lecture PDFs of the vault, to make an exam from one of them.
    OpenSlides,
    /// The lecture PDFs listed (or why they could not be).
    Slides(Result<Vec<SlideChoice>, String>),
    /// Make the exam from the lecture PDF at this vault path.
    FromSlide(String),
    /// The exam was made (or why it was not).
    Generated(Result<Box<Generated>, String>),
    /// Save the exam as a note.
    Save,
    /// The note was saved (its path) or not (why).
    Saved(Result<String, String>),
    /// Show or hide the reference answers.
    ToggleAnswers,
    /// Start writing the answers.
    Start,
    /// An answer was written: its question's index and the text.
    Answer(usize, String),
    /// Grade the answers written so far.
    Finish,
    /// The answers were graded by the local model (the keyword check is done at once when no model is used).
    Graded(Vec<grading::Graded>),
    /// Close the exam view.
    Close,
}

/// A lecture PDF of the vault that an exam can be made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideChoice {
    /// Vault-relative path.
    pub path: String,
    /// Its title.
    pub title: String,
}

/// Where an exam's text comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExamSource {
    /// The notes of a folder (the inbox when `None`).
    Notes(Option<String>),
    /// The text of a lecture PDF, by its vault path.
    Slide(String),
}

/// Work for the executor.
#[derive(Debug, Clone, PartialEq)]
pub enum MockExamEffect {
    /// Make the exam from a source, with the model when one is given.
    Generate {
        /// Where the text comes from.
        source: ExamSource,
        /// Questions asked for.
        total: usize,
        /// The exam's title.
        title: String,
        /// The model that writes the questions, if one may see the text.
        provider: Option<Provider>,
    },
    /// List the lecture PDFs of the vault.
    ListSlides,
    /// Grade the answers with the local model; a question the model cannot grade falls back to the keyword check.
    Grade {
        /// The exam.
        exam: Box<MockExam>,
        /// Its reference answers and rubrics.
        keys: Vec<AnswerKey>,
        /// The answers written, one per question.
        answers: Vec<String>,
        /// The local model that judges the answers.
        provider: Provider,
    },
    /// Save the exam as a note in its folder.
    Save {
        /// The folder to save into.
        folder: Option<String>,
        /// The exam's title.
        title: String,
        /// The exam.
        exam: Box<MockExam>,
        /// Its reference answers.
        keys: Vec<AnswerKey>,
    },
}

/// The exam view: the exam made, and whether its answers are shown.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MockExamUi {
    /// The folder the notes came from.
    pub folder: Option<String>,
    /// The exam's title.
    pub title: String,
    /// The exam is being written, or the PDFs are being listed.
    pub loading: bool,
    /// The answers are being graded by the local model.
    pub judging: bool,
    /// The exam, once made.
    pub result: Option<Box<Generated>>,
    /// Why there is no exam.
    pub error: Option<String>,
    /// What the student should know about how the exam was made (for example, that no model is selected).
    pub note: Option<String>,
    /// The lecture PDFs to choose from, when the student is choosing one.
    pub slides: Option<Vec<SlideChoice>>,
    /// The reference answers are shown.
    pub show_answers: bool,
    /// The vault path of the saved note.
    pub saved: Option<String>,
    /// The student is writing the answers.
    pub started: bool,
    /// The answers written, one per question.
    pub answers: Vec<String>,
    /// The graded questions, once the exam is finished.
    pub graded: Option<Vec<grading::Graded>>,
    /// When the exam was started, on the app clock in seconds; the timer counts from it.
    pub started_at: Option<f64>,
    /// Whole seconds of the exam's time used, as the timer last read them.
    pub elapsed_sec: u32,
    /// The answers are locked: submitted, and the result not yet shown.
    pub locked: bool,
}

/// Handle a mock-exam message.
pub fn update(state: State, msg: MockExamMsg) -> (State, Vec<Effect>) {
    match msg {
        MockExamMsg::Open => open::open(state),
        MockExamMsg::OpenSlides => open::open_slides(state),
        MockExamMsg::Slides(result) => (open::slides_listed(state, result), Vec::new()),
        MockExamMsg::FromSlide(path) => open::from_slide(state, path),
        MockExamMsg::Generated(result) => {
            let ui = map_ui(&state, |ui| match result {
                Ok(g) => {
                    let answers = vec![String::new(); g.keys.len()];
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
        MockExamMsg::Answer(i, text) => {
            let ui = map_ui(&state, |ui| {
                if ui.locked {
                    return ui;
                }
                let mut answers = ui.answers.clone();
                if let Some(slot) = answers.get_mut(i) {
                    *slot = text;
                }
                MockExamUi { answers, ..ui }
            });
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
