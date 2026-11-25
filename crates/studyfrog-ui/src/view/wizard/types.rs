//! The first-run setup wizard's state, messages and effects (STORY-0212, TASK-0299). The steps and their order come
//! from `studyfrog_config::wizard`; this file holds what is on screen around them.

use studyfrog_config::wizard::{Step, Wizard};

use crate::host::PlanView;

/// The wizard: whether it is open, where the user is, and what its MiniLM step has read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WizardState {
    /// The wizard is on screen; it replaces the central panel while it is open.
    pub open: bool,
    /// The step and the steps skipped; `None` until the wizard is opened.
    pub flow: Option<Wizard>,
    /// The MiniLM download plan, once the host has given it.
    pub plan: Option<PlanView>,
    /// The MiniLM files are in the models folder, as the last probe found them.
    pub minilm_installed: bool,
    /// The MiniLM download is running.
    pub downloading: bool,
    /// Why the last MiniLM download failed.
    pub download_error: Option<String>,
}

/// The wizard's messages: moves between steps, the MiniLM click, and the answers of the effects.
#[derive(Debug, Clone, PartialEq)]
pub enum WizardMsg {
    /// Open the wizard at its first optional step (first boot, and Settings).
    Start,
    /// Go to the next step.
    Next,
    /// Go to the step before.
    Previous,
    /// Leave the current step as it is.
    Skip,
    /// Show this step.
    Goto(Step),
    /// Close the wizard.
    Finish,
    /// The user clicked the MiniLM download button.
    ConfirmMinilm,
    /// What the probe found.
    Probed(Probe),
    /// The MiniLM download ended: Ok, or why it did not.
    Fetched(Result<(), String>),
}

/// What the probe found: the MiniLM files on disk and the plan the host gives.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    /// The MiniLM files are in the models folder.
    pub minilm_installed: bool,
    /// The download plan, when the host provides one.
    pub plan: Option<PlanView>,
}

/// Work the wizard asks the worker for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardEffect {
    /// Read the MiniLM files and the download plan.
    Probe,
    /// Download MiniLM. Sent only after the user's click.
    FetchMinilm,
}

/// The flow the wizard is on, or the first optional step when it has not been opened.
pub fn flow_of(w: &WizardState) -> Wizard {
    w.flow
        .clone()
        .unwrap_or_else(studyfrog_config::wizard::start)
}

/// The step's name in the control protocol and the log.
pub fn name_of(step: Step) -> &'static str {
    match step {
        Step::Vault => "vault",
        Step::LocalModel => "local_model",
        Step::RemoteProvider => "remote_provider",
        Step::Minilm => "minilm",
        Step::Summary => "summary",
    }
}

/// The step's title on screen.
pub fn title_of(step: Step) -> &'static str {
    match step {
        Step::Vault => "Vault",
        Step::LocalModel => "Local model",
        Step::RemoteProvider => "Remote provider",
        Step::Minilm => "MiniLM",
        Step::Summary => "Summary",
    }
}
