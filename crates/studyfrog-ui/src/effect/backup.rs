//! Backup, restore and profile effects.

/// Backup, restore and profile effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectBackup {
    /// Make a new profile and switch to it.
    CreateProfile(String),
    /// Switch to another profile.
    SwitchProfile(String),
    /// Delete a profile and its settings and keys.
    DeleteProfile(String),
    /// Ask where to put a backup of the vault and write it.
    #[rustfmt::skip]
    Backup { #[doc = "Put the API keys in it."] include_keys: bool },
    /// Ask for a backup file and read what it holds.
    ChooseBackup,
    /// Ask for an empty folder and unpack a backup there.
    Restore {
        /// The backup file.
        archive: String,
        /// Replace the app settings with those of the backup.
        with_settings: bool,
    },
}
