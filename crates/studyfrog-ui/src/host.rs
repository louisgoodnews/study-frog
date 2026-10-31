//! What the application binary supplies to the UI: things that need heavy or system-specific
//! libraries (a language model, native dialogs) and the extensions to offer, injected so the UI
//! crate and its tests do not link them. A missing piece degrades gracefully.

use std::path::PathBuf;

use studyfrog_ext::types::Registry;
use studyfrog_slm::models::Loader;

use crate::state::Extensions;

/// Opens a native "choose a folder" dialog; `None` when the user cancels or no dialog is available.
pub type FolderPicker = fn() -> Option<PathBuf>;

/// Opens a native "choose a file" dialog for files with these extensions (no dot); `None` when the
/// user cancels or no dialog is available.
pub type FilePicker = fn(&[&str]) -> Option<PathBuf>;

/// The pieces the host provides.
#[derive(Clone, Default)]
pub struct Host {
    /// How to open the language-model runtime.
    pub model_loader: Option<Loader>,
    /// The native folder dialog.
    pub folder_picker: Option<FolderPicker>,
    /// The native file dialog.
    pub file_picker: Option<FilePicker>,
    /// Importers and card types registered by extensions.
    pub registry: Registry,
    /// Sends the AI agent's requests (the application links the network library).
    pub transport: Option<studyfrog_agent::provider::Transport>,
}

/// What the user can choose from the registry, as plain text for the views.
pub fn extensions_of(r: &Registry) -> Extensions {
    Extensions {
        card_types: r
            .card_types
            .iter()
            .map(|c| (c.id.to_owned(), c.name.to_owned(), c.hint.to_owned()))
            .collect(),
        importers: r
            .importers
            .iter()
            .map(|i| {
                (
                    i.id.to_owned(),
                    i.name.to_owned(),
                    i.extensions.iter().map(|e| (*e).to_owned()).collect(),
                )
            })
            .collect(),
    }
}
