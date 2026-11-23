//! Session modal for selecting mock-exam item formats and mix before generation (STORY-0207). Pure.

use studyfrog_rehearsal::exam_format::{
    ExamFormatMix, ExamItemFormat, MixMode, default_format_mix, format_index, resolved_mix_total,
    validate_format_mix,
};

use super::open::exam_model;
use super::{ExamSource, MockExamEffect, MockExamUi, with_ui};
use crate::effect::Effect;
use crate::modal_types::Modal;
use crate::state::State;

/// Configuration dialog state before generating a mock exam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockExamFormatModal {
    /// Target stack folder (for saving notes).
    pub folder: Option<String>,
    /// Notes folder or lecture PDF source.
    pub source: ExamSource,
    /// Title of the exam to generate.
    pub title: String,
    /// Active format mix being edited.
    pub mix: ExamFormatMix,
    /// Inline validation error, if any.
    pub error: Option<String>,
}

/// User actions inside the format selection modal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatModalMsg {
    /// Toggle a format row on or off.
    ToggleFormat(ExamItemFormat, bool),
    /// Set exact count or relative weight for one format.
    SetValue(ExamItemFormat, usize),
    /// Increment or decrement one format's count or weight.
    StepValue(ExamItemFormat, isize),
    /// Switch between proportional weights and exact counts.
    SetMode(MixMode),
    /// Set total question count (in weights mode).
    SetTotal(usize),
    /// Increment or decrement total question count (in weights mode).
    StepTotal(isize),
    /// Restore default 6-format preferences.
    ResetDefaults,
    /// Validate, remember session prefs, close modal, and start generating.
    Confirm,
}

/// Build a modal for generating an exam from stack notes.
pub fn modal_for_notes(
    folder: Option<String>,
    title: String,
    mix: ExamFormatMix,
) -> MockExamFormatModal {
    let error = validate_format_mix(&mix).err();
    MockExamFormatModal {
        folder: folder.clone(),
        source: ExamSource::Notes(folder),
        title,
        mix,
        error,
    }
}

/// Build a modal for generating an exam from a lecture PDF.
pub fn modal_for_slide(path: String, title: String, mix: ExamFormatMix) -> MockExamFormatModal {
    let error = validate_format_mix(&mix).err();
    MockExamFormatModal {
        folder: None,
        source: ExamSource::Slide(path),
        title,
        mix,
        error,
    }
}

/// Handle a [`FormatModalMsg`].
pub fn update(state: State, msg: FormatModalMsg) -> (State, Vec<Effect>) {
    let Some(Modal::MockExamFormat(modal)) = state.modal.clone() else {
        return (state, Vec::new());
    };
    let mut modal = *modal;
    match msg {
        FormatModalMsg::ToggleFormat(fmt, on) => {
            modal.mix.formats[format_index(fmt)].enabled = on;
            if on && modal.mix.formats[format_index(fmt)].value == 0 {
                modal.mix.formats[format_index(fmt)].value = 1;
            }
            modal.error = validate_format_mix(&modal.mix).err();
            with_modal(state, modal)
        }
        FormatModalMsg::SetValue(fmt, val) => {
            modal.mix.formats[format_index(fmt)].value = val.min(50);
            modal.error = validate_format_mix(&modal.mix).err();
            with_modal(state, modal)
        }
        FormatModalMsg::StepValue(fmt, delta) => {
            let cur = modal.mix.formats[format_index(fmt)].value as isize;
            modal.mix.formats[format_index(fmt)].value = (cur + delta).clamp(0, 50) as usize;
            modal.error = validate_format_mix(&modal.mix).err();
            with_modal(state, modal)
        }
        FormatModalMsg::SetMode(mode) => {
            if modal.mix.mode != mode {
                if mode == MixMode::Counts {
                    modal.mix.total_questions = resolved_mix_total(&modal.mix);
                }
                modal.mix.mode = mode;
                modal.error = validate_format_mix(&modal.mix).err();
            }
            with_modal(state, modal)
        }
        FormatModalMsg::SetTotal(total) => {
            modal.mix.total_questions = total.min(50);
            modal.error = validate_format_mix(&modal.mix).err();
            with_modal(state, modal)
        }
        FormatModalMsg::StepTotal(delta) => {
            let cur = modal.mix.total_questions as isize;
            modal.mix.total_questions = (cur + delta).clamp(1, 50) as usize;
            modal.error = validate_format_mix(&modal.mix).err();
            with_modal(state, modal)
        }
        FormatModalMsg::ResetDefaults => {
            modal.mix = default_format_mix();
            modal.error = validate_format_mix(&modal.mix).err();
            with_modal(state, modal)
        }
        FormatModalMsg::Confirm => confirm(state, modal),
    }
}

fn with_modal(state: State, modal: MockExamFormatModal) -> (State, Vec<Effect>) {
    (
        State {
            modal: Some(Modal::MockExamFormat(Box::new(modal))),
            ..state
        },
        Vec::new(),
    )
}

fn confirm(state: State, mut modal: MockExamFormatModal) -> (State, Vec<Effect>) {
    if let Err(e) = validate_format_mix(&modal.mix) {
        modal.error = Some(e);
        return with_modal(state, modal);
    }
    let (provider, note) = exam_model(&state);
    let mut next = State {
        modal: None,
        rehearse: crate::setup::Rehearse {
            mock_format_prefs: modal.mix.clone(),
            ..state.rehearse.clone()
        },
        ..state
    };
    let ui = MockExamUi {
        folder: modal.folder,
        title: modal.title.clone(),
        loading: true,
        note,
        format_mix: modal.mix.clone(),
        ..MockExamUi::default()
    };
    next = with_ui(next, Some(ui));
    let effect = Effect::MockExam(MockExamEffect::Generate {
        source: modal.source,
        mix: modal.mix,
        title: modal.title,
        provider,
    });
    (next, vec![effect])
}
