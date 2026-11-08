//! Making a worker: with nothing open, or with what the host provides.

use std::path::PathBuf;

use studyfrog_ext::types::Registry;

use crate::exec::Worker;
use crate::host::Host;

/// A worker with nothing open.
pub fn worker_new(config_dir: Option<PathBuf>) -> Worker {
    Worker {
        root_config: config_dir.clone(),
        config_dir,
        session: None,
        models_dir: None,
        slm: None,
        loader: None,
        folder_picker: None,
        file_picker: None,
        registry: Registry::default(),
        transport: None,
        state_dir: None,
        pdf_bytes: None,
        opener: None,
    }
}

/// A worker that looks for language models in `models_dir` and uses what the host provides.
pub fn worker_with_host(
    config_dir: Option<PathBuf>,
    models_dir: Option<PathBuf>,
    host: Host,
) -> Worker {
    Worker {
        models_dir,
        loader: host.model_loader,
        folder_picker: host.folder_picker,
        file_picker: host.file_picker,
        registry: host.registry,
        transport: host.transport,
        opener: host.opener,
        ..worker_new(config_dir)
    }
}
