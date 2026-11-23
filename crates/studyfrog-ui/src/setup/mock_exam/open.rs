//! Opening a mock exam (STORY-0186, STORY-0207): validates the source and opens the format selection modal.

use studyfrog_agent::provider::{Provider, may_send};

use super::format_modal::{modal_for_notes, modal_for_slide};
use super::{MockExamEffect, MockExamUi, SlideChoice, with_ui};
use crate::agent_conn::selected_provider;
use crate::effect::Effect;
use crate::modal_types::Modal;
use crate::pdf_names::file_name;
use crate::setup::stack_source::{StackSource, stack_source};
use crate::state::State;

/// The note when no model is selected: templates write the questions.
pub const NO_MODEL: &str = "No model is selected, so the questions come from templates. Install a local model and select it in Agent settings to have it write them from your notes or slides.";
/// The note when the selected model is remote and may not see the text.
pub const REMOTE_NOT_ALLOWED: &str = "The selected model is remote and may not see your notes, so the questions come from templates. Allow it in Agent settings, or select a local model.";

/// Validate the open stack and open the item format configuration modal.
pub fn open(state: State) -> (State, Vec<Effect>) {
    let folder = match stack_source(&state) {
        StackSource::Stack(f) => Some(f),
        StackSource::Inbox => None,
        StackSource::Nothing => {
            let ui = MockExamUi {
                error: Some(
                    "Tick a stack in the What step, or open one in the library, then make its exam."
                        .to_owned(),
                ),
                ..MockExamUi::default()
            };
            return (with_ui(state, Some(ui)), Vec::new());
        }
    };
    let title = match &folder {
        Some(f) => format!("Mock exam: {f}"),
        None => "Mock exam: inbox".to_owned(),
    };
    let modal = modal_for_notes(folder, title, state.rehearse.mock_format_prefs.clone());
    (
        State {
            modal: Some(Modal::MockExamFormat(Box::new(modal))),
            ..state
        },
        Vec::new(),
    )
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
        with_ui(state, Some(ui)),
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
    with_ui(state, Some(ui))
}

/// Open the item format configuration modal for the lecture PDF at `path`.
pub fn from_slide(state: State, path: String) -> (State, Vec<Effect>) {
    let title = format!("Mock exam: {}", file_name(&path));
    let modal = modal_for_slide(path, title, state.rehearse.mock_format_prefs.clone());
    (
        State {
            modal: Some(Modal::MockExamFormat(Box::new(modal))),
            ..state
        },
        Vec::new(),
    )
}

/// The model that writes the questions, and the note for the student when there is none.
pub fn exam_model(state: &State) -> (Option<Provider>, Option<String>) {
    match selected_provider(&state.agent) {
        None => (None, Some(NO_MODEL.to_owned())),
        Some(p) if may_send(&p) => (Some(p), None),
        Some(_) => (None, Some(REMOTE_NOT_ALLOWED.to_owned())),
    }
}
