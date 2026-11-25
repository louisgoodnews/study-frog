//! What the application binary supplies to the UI: things that need heavy or system-specific
//! libraries (a language model, native dialogs) and the extensions to offer, injected so the UI
//! crate and its tests do not link them. A missing piece degrades gracefully.

use std::path::{Path, PathBuf};

use studyfrog_ext::types::Registry;
use studyfrog_slm::models::Loader;

use crate::state::Extensions;

/// Opens a native "choose a folder" dialog; `None` when the user cancels or no dialog is available.
pub type FolderPicker = fn() -> Option<PathBuf>;

/// Opens a native "choose a file" dialog for files with these extensions (no dot); `None` when the
/// user cancels or no dialog is available.
pub type FilePicker = fn(&[&str]) -> Option<PathBuf>;

/// Shows a folder in the system's file manager; the error is a sentence for the user.
pub type Opener = fn(&std::path::Path) -> Result<(), String>;

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
    /// Fetches the local Ollama status (the application links the network library).
    pub get: Option<studyfrog_agent::provider::HttpGet>,
    /// Shows a folder in the system's file manager (the logs folder from Settings > Data).
    pub opener: Option<Opener>,
    /// The setup wizard's model download (the application links the network library).
    pub models: Option<ModelHost>,
}

/// The model download the setup wizard offers, supplied by the application (STORY-0212).
#[derive(Debug, Clone, Copy)]
pub struct ModelHost {
    /// The MiniLM download as the wizard shows it: title, source and files, before the user's click.
    pub minilm_plan: fn() -> PlanView,
    /// Downloads MiniLM into this folder when `confirmed` is true; nothing is contacted otherwise.
    pub fetch_minilm: fn(&Path, bool) -> Result<(), String>,
}

/// A download as the wizard shows it: what it is, where it comes from and its files with their sizes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanView {
    /// Human-readable title.
    pub title: String,
    /// The host the files are fetched from.
    pub source_host: String,
    /// File name and expected size in bytes, in download order.
    pub files: Vec<(String, u64)>,
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
