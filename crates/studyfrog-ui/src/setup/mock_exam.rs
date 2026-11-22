//! The mock exam from a folder's notes (STORY-0186): asking for it, showing what was made, and saving it
//! as a note. The generation is in `studyfrog-service::exam_gen`. Pure: no egui, no I/O.

use studyfrog_agent::provider::Provider;
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::mock_exam::MockExam;
use studyfrog_service::exam_gen::Generated;

use crate::agent_conn::selected_provider;
use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, Selection, State};

/// Questions asked for an exam from the notes.
pub const EXAM_QUESTIONS: usize = 10;

/// Messages of the mock exam.
#[derive(Debug, Clone, PartialEq)]
pub enum MockExamMsg {
    /// Make an exam from the stack that is open.
    Open,
    /// The exam was made (or why it was not).
    Generated(Result<Box<Generated>, String>),
    /// Save the exam as a note.
    Save,
    /// The note was saved (its path) or not (why).
    Saved(Result<String, String>),
    /// Show or hide the reference answers.
    ToggleAnswers,
    /// Close the exam view.
    Close,
}

/// Work for the executor.
#[derive(Debug, Clone, PartialEq)]
pub enum MockExamEffect {
    /// Make the exam from the notes of a folder (the inbox when `None`), with the model when one is given.
    Generate {
        /// The folder whose notes are read.
        folder: Option<String>,
        /// Questions asked for.
        total: usize,
        /// The exam's title.
        title: String,
        /// The model that writes the questions, if one may see the notes.
        provider: Option<Provider>,
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
    /// The exam is being written.
    pub loading: bool,
    /// The exam, once made.
    pub result: Option<Box<Generated>>,
    /// Why there is no exam.
    pub error: Option<String>,
    /// The reference answers are shown.
    pub show_answers: bool,
    /// The vault path of the saved note.
    pub saved: Option<String>,
}

/// Handle a mock-exam message.
pub fn update(state: State, msg: MockExamMsg) -> (State, Vec<Effect>) {
    match msg {
        MockExamMsg::Open => open(state),
        MockExamMsg::Generated(result) => {
            let ui = map_ui(&state, |ui| match result {
                Ok(g) => MockExamUi {
                    loading: false,
                    result: Some(g),
                    error: None,
                    ..ui
                },
                Err(e) => MockExamUi {
                    loading: false,
                    error: Some(e),
                    ..ui
                },
            });
            (with_ui(state, ui), Vec::new())
        }
        MockExamMsg::Save => save(state),
        MockExamMsg::Saved(result) => saved(state, result),
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

fn open(state: State) -> (State, Vec<Effect>) {
    let folder = match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        Selection::Inbox => None,
        Selection::Nothing => {
            let ui = MockExamUi {
                error: Some("Open a stack first, then make its exam.".to_owned()),
                ..MockExamUi::default()
            };
            return (with_ui(state, Some(ui)), Vec::new());
        }
    };
    let title = match &folder {
        Some(f) => format!("Mock exam: {f}"),
        None => "Mock exam: inbox".to_owned(),
    };
    // A model sees the notes only when it is local, or when the user allowed it.
    let provider = selected_provider(&state.agent).filter(studyfrog_agent::provider::may_send);
    let ui = MockExamUi {
        folder: folder.clone(),
        title: title.clone(),
        loading: true,
        ..MockExamUi::default()
    };
    let effect = Effect::MockExam(MockExamEffect::Generate {
        folder,
        total: EXAM_QUESTIONS,
        title,
        provider,
    });
    (with_ui(state, Some(ui)), vec![effect])
}

fn save(state: State) -> (State, Vec<Effect>) {
    let Some(ui) = state.rehearse.mock.clone() else {
        return (state, Vec::new());
    };
    let Some(g) = ui.result.as_ref() else {
        return (state, Vec::new());
    };
    let effect = Effect::MockExam(MockExamEffect::Save {
        folder: ui.folder.clone(),
        title: ui.title.clone(),
        exam: Box::new(g.exam.clone()),
        keys: g.keys.clone(),
    });
    (state, vec![effect])
}

fn saved(state: State, result: Result<String, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(path) => {
            let ui = map_ui(&state, |ui| MockExamUi {
                saved: Some(path.clone()),
                ..ui
            });
            let state = with_ui(state, ui);
            let notice = Notice {
                kind: NoticeKind::Info,
                text: format!("The exam was saved as a note: {path}"),
            };
            (
                State {
                    notice: Some(notice),
                    ..state
                },
                Vec::new(),
            )
        }
        Err(e) => {
            let notice = Notice {
                kind: NoticeKind::Error,
                text: format!("The exam could not be saved: {e}"),
            };
            (
                State {
                    notice: Some(notice),
                    ..state
                },
                Vec::new(),
            )
        }
    }
}
