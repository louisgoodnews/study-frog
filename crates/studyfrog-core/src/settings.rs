//! Settings as plain data. They are read from TOML files (`studyfrog.toml` at the vault root and
//! `_stack.toml` in a stack folder) by `studyfrog-config`, never from Markdown front-matter.

use crate::scheduler::Scheduler;

/// Per-stack settings from `_stack.toml`. Every field is optional: `None` means "use the vault default".
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StackSettings {
    /// Scheduling algorithm.
    pub scheduler: Option<Scheduler>,
    /// FSRS target retention in `(0, 1)`.
    pub request_retention: Option<f32>,
    /// New items introduced per day.
    pub daily_new_limit: Option<u32>,
    /// Reviews per day.
    pub daily_review_limit: Option<u32>,
}

/// Vault-wide settings from `studyfrog.toml`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VaultSettings {
    /// Default scheduler for stacks that do not choose one.
    pub scheduler: Scheduler,
    /// Default FSRS target retention in `(0, 1)`.
    pub request_retention: f32,
    /// Upper bound for any interval, in days.
    pub maximum_interval_days: u32,
    /// Custom FSRS weights; `None` means the published defaults.
    pub weights: Option<[f32; 21]>,
}

/// Defaults used when `studyfrog.toml` is absent or leaves a key out.
pub fn vault_settings_default() -> VaultSettings {
    VaultSettings {
        scheduler: Scheduler::Fsrs,
        request_retention: 0.9,
        maximum_interval_days: 36500,
        weights: None,
    }
}
