//! The application binary's own pieces: the native dialog. Depends on studyfrog-ui and the optional
//! heavy runtimes; nothing else depends on this crate.

pub mod picker;

use studyfrog_ui::host::Host;

/// What this binary provides to the UI.
pub fn host() -> Host {
    Host {
        model_loader: Some(studyfrog_candle::minilm_loader),
        folder_picker: Some(picker::pick_folder),
    }
}
