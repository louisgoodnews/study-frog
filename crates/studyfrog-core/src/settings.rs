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

/// Most recent vaults remembered by the app.
pub const MAX_RECENT_VAULTS: usize = 8;

/// Slowest accepted reading speed, in words per minute.
pub const MIN_READING_WPM: u32 = 50;
/// Fastest accepted reading speed, in words per minute.
pub const MAX_READING_WPM: u32 = 1000;

/// Per-user application settings (not part of any vault): where the app last worked.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppSettings {
    /// The vault opened last; the app reopens it at start-up.
    pub vault: Option<String>,
    /// Recently opened vaults, newest first, without duplicates.
    pub recent: Vec<String>,
    /// Measured reading speed in words per minute; `None` means the built-in default is used.
    pub reading_wpm: Option<u32>,
}

/// `settings` after opening `path`: it becomes the current vault and moves to the front of the recent list.
pub fn with_opened_vault(settings: &AppSettings, path: &str) -> AppSettings {
    let mut recent: Vec<String> = std::iter::once(path.to_owned())
        .chain(
            settings
                .recent
                .iter()
                .filter(|p| p.as_str() != path)
                .cloned(),
        )
        .collect();
    recent.truncate(MAX_RECENT_VAULTS);
    AppSettings {
        vault: Some(path.to_owned()),
        recent,
        ..settings.clone()
    }
}
