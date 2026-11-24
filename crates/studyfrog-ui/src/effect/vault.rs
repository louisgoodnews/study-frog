//! Vault, settings, reminders and subject book effects.

use studyfrog_core::settings::AppSettings;

/// Vault, settings, reminders and subject book effects.
#[derive(Debug, Clone, PartialEq)]
// The largest payload is a rare save or import outcome; boxing it would change every call site.
#[allow(clippy::large_enum_variant)]
pub enum EffectVault {
    /// Read the per-user app settings.
    LoadAppSettings,
    /// Ask the user for a vault folder with the native dialog.
    PickVaultFolder,
    /// Open (and index) a vault folder.
    OpenVault(String),
    /// Check the open vault for changed files.
    PollVault,
    /// Look for calendar and homework reminders that have come due.
    CheckReminders,
    /// Read the reminders due now without marking them shown (`reminders due`).
    PeekReminders,
    /// Persist the per-user app settings.
    SaveAppSettings(AppSettings),
    /// Write the calibrated-start switch into the vault's `studyfrog.toml`.
    SetCalibratedStart(bool),
    /// Read what the home screen shows.
    LoadHome,
    /// Read the vault's teachers and subjects.
    LoadSubjects,
    /// Store the vault's teachers and subjects.
    SaveSubjects(studyfrog_config::subjects::SubjectBook),
}
