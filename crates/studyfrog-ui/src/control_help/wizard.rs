//! The command table rows of the setup wizard (STORY-0212): its status and its steps. A step moves the wizard only;
//! the MiniLM download needs the button on the screen, never a command.

use crate::control_help::Row;

/// Rows for the setup wizard.
pub const ROWS: &[Row] = &[
    (
        "wizard | wizard status",
        "Setup wizard: whether it is open, its current step (local_model, remote_provider, minilm, summary) and each step's status (current, done, skipped, pending)",
        "read",
    ),
    (
        "wizard step next | previous | skip",
        "Setup wizard: go to the next or previous step, or skip the current one; no step starts a download",
        "write",
    ),
    (
        "wizard step goto <step>",
        "Setup wizard: show this step (local_model, remote_provider, minilm, summary)",
        "write",
    ),
];
