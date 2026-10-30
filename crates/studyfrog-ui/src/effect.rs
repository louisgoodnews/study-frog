//! Side effects `update` asks for, as plain data. The executor performs them and answers with a `Msg`.

use studyfrog_core::settings::AppSettings;

/// Work to do outside the pure update.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Read the per-user app settings.
    LoadAppSettings,
    /// Open (and index) a vault folder.
    OpenVault(String),
    /// Check the open vault for changed files.
    PollVault,
    /// Persist the per-user app settings.
    SaveAppSettings(AppSettings),
}
