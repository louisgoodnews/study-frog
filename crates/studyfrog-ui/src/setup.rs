//! Setup screen state and its update logic. The form itself is in [`crate::setup_form`].
//! Pure: no egui, no I/O.

use crate::effect::Effect;
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::state::State;

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
