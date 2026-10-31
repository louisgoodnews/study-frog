//! The application binary's own pieces: the native dialogs and the choice of extensions. Depends on
//! studyfrog-ui and the optional heavy runtimes; nothing else depends on this crate.

pub mod picker;

use studyfrog_ext::registry::registry_new;
use studyfrog_ui::host::Host;

/// What this binary provides to the UI. The one line of wiring for an extension is its `register`
/// call; an extension that fails to register is reported and left out.
pub fn host() -> Host {
    let mut registry = registry_new();
    if let Err(e) = studyfrog_ext_example::register(&mut registry) {
        eprintln!("studyfrog: the example extension was not loaded: {e}");
    }
    Host {
        model_loader: Some(studyfrog_candle::minilm_loader),
        folder_picker: Some(picker::pick_folder),
        file_picker: Some(picker::pick_file),
        registry,
        transport: Some(studyfrog_net::send),
    }
}
