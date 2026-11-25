//! The setup wizard's commands of the control protocol (STORY-0212): `wizard status` and `wizard step ...`. A step
//! moves the wizard only; no command starts a download, which needs the user's own click.

use studyfrog_config::wizard::Step;

/// What `wizard ...` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardCmd {
    /// Whether the wizard is open, the step it is on and each step's status.
    Status,
    /// Move the wizard through its steps.
    Step(StepOp),
}

/// A move of the wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOp {
    /// The next step.
    Next,
    /// The step before.
    Previous,
    /// Leave the current step as it is.
    Skip,
    /// Show this step.
    Goto(Step),
}
