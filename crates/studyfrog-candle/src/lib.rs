//! The semantic runtime for answer comparison, kept in its own crate so that only the application
//! binary links the tensor library; every other crate and test stays fast to build and link.

pub mod minilm;

use std::path::Path;

use studyfrog_slm::runtime::{SlmError, SlmRuntime};

/// The loader the application hands to the UI: opens the MiniLM model in `dir`.
pub fn minilm_loader(dir: &Path) -> Result<Box<dyn SlmRuntime>, SlmError> {
    minilm::minilm_load(dir).map(|m| Box::new(m) as Box<dyn SlmRuntime>)
}
