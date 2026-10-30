//! What the application binary supplies to the UI: things that need heavy or system-specific
//! libraries (a language model, a native dialog), injected so the UI crate and its tests do not
//! link them. A missing piece degrades gracefully.

use std::path::PathBuf;

use studyfrog_slm::models::Loader;

/// Opens a native "choose a folder" dialog; `None` when the user cancels or no dialog is available.
pub type FolderPicker = fn() -> Option<PathBuf>;

/// The pieces the host provides.
#[derive(Clone, Copy, Default)]
pub struct Host {
    /// How to open the language-model runtime.
    pub model_loader: Option<Loader>,
    /// The native folder dialog.
    pub folder_picker: Option<FolderPicker>,
}
