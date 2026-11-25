//! The first-run setup wizard: its optional steps, the step the user is on and the steps they skipped
//! (STORY-0212, TASK-0297). Pure: the caller reports what is installed; nothing here touches files, the network or
//! the screen.

/// A step of the wizard, in order. The vault is chosen before the wizard starts, so it is shown as done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The vault, chosen before the wizard starts.
    Vault,
    /// A local model through Ollama.
    LocalModel,
    /// A remote model provider (opt-in).
    RemoteProvider,
    /// The MiniLM embedding model.
    Minilm,
    /// The summary of what was set up and skipped.
    Summary,
}

/// The steps in order.
pub const STEPS: [Step; 5] = [
    Step::Vault,
    Step::LocalModel,
    Step::RemoteProvider,
    Step::Minilm,
    Step::Summary,
];

/// What is already set up, as reported by the caller.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Installed {
    /// A vault is chosen.
    pub vault: bool,
    /// A local model is installed and usable.
    pub local_model: bool,
    /// A remote provider is configured.
    pub remote_provider: bool,
    /// The MiniLM model is installed.
    pub minilm: bool,
}

/// Where the user is in the wizard: the position in [`STEPS`] and the steps they skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wizard {
    position: usize,
    skipped: Vec<Step>,
}

/// How a step is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The step the wizard is on.
    Current,
    /// Already set up.
    Done,
    /// The user skipped it.
    Skipped,
    /// Not yet visited and not set up.
    Pending,
}

/// The wizard at its first optional step, after the vault. Used at first boot and when reopened from Settings.
pub fn start() -> Wizard {
    Wizard {
        position: 1,
        skipped: Vec::new(),
    }
}

/// The step the wizard is on.
pub fn current(w: &Wizard) -> Step {
    STEPS.get(w.position).copied().unwrap_or(Step::Summary)
}

/// True once the wizard has reached its summary.
pub fn is_finished(w: &Wizard) -> bool {
    current(w) == Step::Summary
}

/// The wizard one step further on; the summary is the last step.
pub fn next(w: &Wizard) -> Wizard {
    Wizard {
        position: (w.position + 1).min(last_position()),
        skipped: w.skipped.clone(),
    }
}

/// The wizard one step back. The vault is not revisited, so the first optional step is the earliest.
pub fn previous(w: &Wizard) -> Wizard {
    Wizard {
        position: w.position.saturating_sub(1).max(1),
        skipped: w.skipped.clone(),
    }
}

/// The wizard after skipping the current step. The vault and the summary cannot be skipped.
pub fn skip(w: &Wizard) -> Wizard {
    let step = current(w);
    if matches!(step, Step::Vault | Step::Summary) {
        return w.clone();
    }
    let mut skipped = w.skipped.clone();
    if !skipped.contains(&step) {
        skipped.push(step);
    }
    next(&Wizard {
        position: w.position,
        skipped,
    })
}

/// The wizard moved to `step` (to revisit a step, for example one that is already done). The vault cannot be
/// revisited, so it leaves the wizard where it is.
pub fn goto(w: &Wizard, step: Step) -> Wizard {
    if step == Step::Vault {
        return w.clone();
    }
    Wizard {
        position: position_of(step),
        skipped: w.skipped.clone(),
    }
}

/// How `step` is shown: the current step first, then done when already set up, then skipped, else pending.
pub fn status(step: Step, installed: &Installed, w: &Wizard) -> Status {
    if step == current(w) {
        Status::Current
    } else if satisfied(step, installed) {
        Status::Done
    } else if w.skipped.contains(&step) {
        Status::Skipped
    } else {
        Status::Pending
    }
}

fn satisfied(step: Step, installed: &Installed) -> bool {
    match step {
        Step::Vault => installed.vault,
        Step::LocalModel => installed.local_model,
        Step::RemoteProvider => installed.remote_provider,
        Step::Minilm => installed.minilm,
        Step::Summary => false,
    }
}

fn position_of(step: Step) -> usize {
    STEPS.iter().position(|s| *s == step).unwrap_or(0)
}

fn last_position() -> usize {
    STEPS.len() - 1
}
