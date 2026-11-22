//! Opening a mock exam (STORY-0186): from the notes of the stack that is open, or from a lecture PDF of the
//! vault (its slides). The model writes the questions when one is selected and may see the text. Pure.

use studyfrog_agent::provider::{Provider, may_send};

use crate::agent_conn::selected_provider;
use crate::effect::Effect;
use crate::setup::mock_exam::{
    EXAM_QUESTIONS, ExamSource, MockExamEffect, MockExamUi, SlideChoice,
};
use crate::state::{Selection, State};

/// The note when no model is selected: templates write the questions.
pub const NO_MODEL: &str = "No model is selected, so the questions come from templates. Install a local model and select it in Agent settings to have it write them from your notes or slides.";
/// The note when the selected model is remote and may not see the text.
pub const REMOTE_NOT_ALLOWED: &str = "The selected model is remote and may not see your notes, so the questions come from templates. Allow it in Agent settings, or select a local model.";

/// Make an exam from the stack that is open (or the inbox).
pub fn open(state: State) -> (State, Vec<Effect>) {
    let folder = match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        Selection::Inbox => None,
        Selection::Nothing => {
            let ui = MockExamUi {
                error: Some("Open a stack first, then make its exam.".to_owned()),
                ..MockExamUi::default()
            };
            return (super::with_ui(state, Some(ui)), Vec::new());
        }
    };
    let title = match &folder {
        Some(f) => format!("Mock exam: {f}"),
        None => "Mock exam: inbox".to_owned(),
    };
    let (provider, note) = exam_model(&state);
    let ui = MockExamUi {
        folder: folder.clone(),
        title: title.clone(),
        loading: true,
        note,
        ..MockExamUi::default()
    };
    let effect = Effect::MockExam(MockExamEffect::Generate {
        source: ExamSource::Notes(folder),
        total: EXAM_QUESTIONS,
        title,
        provider,
    });
    (super::with_ui(state, Some(ui)), vec![effect])
}

/// Ask the vault for its lecture PDFs, to choose the one an exam is made from.
pub fn open_slides(state: State) -> (State, Vec<Effect>) {
    let (_, note) = exam_model(&state);
    let ui = MockExamUi {
        title: "Mock exam from a lecture PDF".to_owned(),
        loading: true,
        note,
        ..MockExamUi::default()
    };
    (
        super::with_ui(state, Some(ui)),
        vec![Effect::MockExam(MockExamEffect::ListSlides)],
    )
}

/// The PDFs listed (or why they could not be). Nothing changes when the exam view was closed meanwhile.
pub fn slides_listed(state: State, result: Result<Vec<SlideChoice>, String>) -> State {
    let Some(ui) = state.rehearse.mock.clone() else {
        return state;
    };
    let ui = match result {
        Ok(list) => MockExamUi {
            loading: false,
            slides: Some(list),
            ..ui
        },
        Err(e) => MockExamUi {
            loading: false,
            error: Some(e),
            ..ui
        },
    };
    super::with_ui(state, Some(ui))
}

/// Make an exam from the lecture PDF at `path`.
pub fn from_slide(state: State, path: String) -> (State, Vec<Effect>) {
    let title = state
        .rehearse
        .mock
        .as_ref()
        .and_then(|ui| ui.slides.as_ref())
        .and_then(|list| list.iter().find(|c| c.path == path))
        .map_or_else(
            || format!("Mock exam: {path}"),
            |c| format!("Mock exam: {}", c.title),
        );
    let (provider, note) = exam_model(&state);
    let ui = MockExamUi {
        title: title.clone(),
        loading: true,
        note,
        ..MockExamUi::default()
    };
    let effect = Effect::MockExam(MockExamEffect::Generate {
        source: ExamSource::Slide(path),
        total: EXAM_QUESTIONS,
        title,
        provider,
    });
    (super::with_ui(state, Some(ui)), vec![effect])
}

/// The model that writes the questions, and the note for the student when there is none. A model sees the
/// text only when it is local, or when the user allowed it.
pub fn exam_model(state: &State) -> (Option<Provider>, Option<String>) {
    match selected_provider(&state.agent) {
        None => (None, Some(NO_MODEL.to_owned())),
        Some(p) if may_send(&p) => (Some(p), None),
        Some(_) => (None, Some(REMOTE_NOT_ALLOWED.to_owned())),
    }
}
