//! Side effects `update` asks for, as plain data. The executor performs them and answers with a `Msg`.

use studyfrog_core::settings::AppSettings;

use crate::state::Selection;

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
    /// List the items of a stack (or the inbox).
    LoadItems(Selection),
    /// Create a stack with this title.
    CreateStack(String),
    /// Create a card in the stack at `folder`.
    CreateCard {
        /// Vault-relative stack folder.
        folder: String,
        /// Title (may be empty).
        title: String,
        /// Front text.
        front: String,
        /// Back text.
        back: String,
    },
}
