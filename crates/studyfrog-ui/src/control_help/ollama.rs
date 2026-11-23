//! The command table rows of the local-model setup (STORY-0206): the audit, Ollama's status and the
//! suggestions. None of them installs, downloads or starts anything.

use crate::control_help::Row;

/// Rows for the audit and the local Ollama service.
pub const ROWS: &[Row] = &[
    (
        "audit",
        "Agent: checks this computer (processor, memory, graphics cards, free disk for the models) and answers with it; a value that cannot be read is null",
        "read",
    ),
    (
        "ollama | ollama status",
        "Agent: asks the local Ollama service (127.0.0.1:11434 only) for its state (missing, not_running, no_models, ready), version and installed models, and the setup steps",
        "read",
    ),
    (
        "ollama suggest",
        "Agent: the models suggested for this computer, ranked by fit (fits, fit, download and memory bytes, installed, reason)",
        "read",
    ),
];
