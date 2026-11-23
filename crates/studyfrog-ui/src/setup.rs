//! Setup screen state and its update logic. The form itself is in [`crate::setup_form`].
//! Pure: no egui, no I/O.

use crate::effect::Effect;
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::state::State;

pub mod case;
pub mod math;
pub mod memory;
pub mod memory_deck;
pub mod memory_flip;
pub mod memory_layout;
pub mod mock_exam;
pub mod stack_source;

pub use crate::setup_form::{
    Preset, SetupEdit, SetupForm, apply_edit, form_for_preset, plan_from_form,
};

/// What the preview line says.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Preview {
    /// Not requested yet.
    #[default]
    Unknown,
    /// Being counted.
    Counting,
    /// This many items match.
    Count(usize),
    /// The form is not valid (reason).
    Invalid(String),
    /// Counting failed (reason).
    Failed(String),
}

/// The three steps of the setup: what to rehearse, how, and a last look before starting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupStep {
    /// Choose the stacks (and kinds, tags).
    #[default]
    What,
    /// Configure the run.
    How,
    /// Review the settings and begin.
    Review,
}

/// State of the rehearse screen's setup.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rehearse {
    /// The form.
    pub form: SetupForm,
    /// The step on screen.
    pub step: SetupStep,
    /// Items that would be rehearsed.
    pub preview: Preview,
    /// A run is being created.
    pub starting: bool,
    /// The run on screen, if any.
    pub run: Option<RunView>,
    /// The memory game on screen, if one was started.
    pub memory: Option<memory::MemoryUi>,
    /// The case-study view, when it is open.
    pub case_study: Option<case::CaseUi>,
    /// The math drill, when it is open.
    pub math: Option<math::MathDrill>,
    /// The mock exam from the notes, when it is open.
    pub mock: Option<mock_exam::MockExamUi>,
    /// The timing chosen for the next memory game.
    pub memory_timer: studyfrog_rehearsal::memory_summary::MemoryTimer,
    /// Last confirmed mock-exam item format mix remembered for this session (STORY-0207).
    pub mock_format_prefs: studyfrog_rehearsal::exam_format::ExamFormatMix,
}

/// Ask for a fresh count (or show why the form is invalid).
pub fn refresh_preview(state: State) -> (State, Vec<Effect>) {
    let form = &state.rehearse.form;
    match plan_from_form(form, 0) {
        Ok(plan) => {
            let effect = Effect::PreviewQueue {
                plan,
                mode: form.mode,
            };
            let rehearse = Rehearse {
                preview: Preview::Counting,
                ..state.rehearse.clone()
            };
            (State { rehearse, ..state }, vec![effect])
        }
        Err(reason) => {
            let rehearse = Rehearse {
                preview: Preview::Invalid(reason),
                ..state.rehearse.clone()
            };
            (State { rehearse, ..state }, Vec::new())
        }
    }
}

/// Handle the setup messages.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::Setup(edit) => {
            let form = apply_edit(state.rehearse.form.clone(), edit);
            refresh_preview(State {
                rehearse: Rehearse {
                    form,
                    ..state.rehearse.clone()
                },
                ..state
            })
        }
        Msg::SetupStep(step) => (
            State {
                rehearse: Rehearse {
                    step,
                    ..state.rehearse.clone()
                },
                ..state
            },
            Vec::new(),
        ),
        Msg::PreviewReady(result) => {
            let preview = match result {
                Ok(n) => Preview::Count(n),
                Err(e) => Preview::Failed(e),
            };
            (
                State {
                    rehearse: Rehearse {
                        preview,
                        ..state.rehearse.clone()
                    },
                    ..state
                },
                Vec::new(),
            )
        }
        _ => (state, Vec::new()),
    }
}

/// The control word of a preset: the words `rehearse setup --preset` takes.
pub fn preset_name(p: Preset) -> &'static str {
    match p {
        Preset::Spaced => "spaced",
        Preset::EverythingInOrder => "all",
        Preset::Shuffled => "shuffled",
        Preset::Hardest => "hardest",
        Preset::Stalest => "stalest",
    }
}

/// The preset the form matches, if it matches one. Stacks, kinds and tags do not count; a field
/// changed away from its preset leaves none.
pub fn preset_of(f: &SetupForm) -> Option<Preset> {
    [
        Preset::Spaced,
        Preset::EverythingInOrder,
        Preset::Shuffled,
        Preset::Hardest,
        Preset::Stalest,
    ]
    .into_iter()
    .find(|p| apply_edit(f.clone(), SetupEdit::Preset(*p)) == *f)
}

/// A new setup starts from Spaced (BUG-0054): the preset is reset, and the stacks, kinds and tags
/// chosen before are kept.
pub fn start_setup(state: State) -> State {
    let form = apply_edit(
        state.rehearse.form.clone(),
        SetupEdit::Preset(Preset::Spaced),
    );
    State {
        rehearse: Rehearse {
            form,
            ..state.rehearse.clone()
        },
        ..state
    }
}
