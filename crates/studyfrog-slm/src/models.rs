//! Where models live and which runtime to use. Nothing is bundled: the user places a model in the
//! app data directory, and without one answers are compared by the built-in lexical runtime.

use std::path::{Path, PathBuf};

use crate::lexical::Lexical;
use crate::runtime::{SlmError, SlmRuntime};

/// The model the semantic runtime expects.
pub const MODEL_NAME: &str = "all-MiniLM-L6-v2";
/// Files a model directory must hold.
pub const MODEL_FILES: [&str; 3] = ["config.json", "tokenizer.json", "model.safetensors"];

/// `$XDG_DATA_HOME/studyfrog/models`, else `$HOME/.local/share/studyfrog/models`.
pub fn models_dir(xdg_data_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    let base = match (xdg_data_home, home) {
        (Some(x), _) if !x.is_empty() => PathBuf::from(x),
        (_, Some(h)) if !h.is_empty() => Path::new(h).join(".local/share"),
        _ => return None,
    };
    Some(base.join("studyfrog").join("models"))
}

/// The files missing from a model directory (all of them when it does not exist).
pub fn missing_files(dir: &Path) -> Vec<&'static str> {
    MODEL_FILES
        .into_iter()
        .filter(|f| !dir.join(f).is_file())
        .collect()
}

/// How to install the model, for messages.
pub fn install_hint(models: &Path) -> String {
    format!(
        "put {} ({}) in {}",
        MODEL_NAME,
        MODEL_FILES.join(", "),
        models.join(MODEL_NAME).display()
    )
}

/// A runtime and the reason the semantic one is not in use (`None` when it is).
pub type Opened = (Box<dyn SlmRuntime>, Option<String>);

/// Opens the semantic runtime from a model directory. Supplied by the application so that this
/// crate does not depend on a tensor library.
pub type Loader = fn(&Path) -> Result<Box<dyn SlmRuntime>, SlmError>;

/// The runtime to use: the semantic one when a loader is available, the model is installed and it
/// loads, else the lexical one. The second value says why the semantic runtime is not in use.
pub fn open_runtime(models: Option<&Path>, loader: Option<Loader>) -> Opened {
    let lexical = |why: String| -> Opened { (Box::new(Lexical), Some(why)) };
    let Some(models) = models else {
        return lexical("no data directory is known".to_owned());
    };
    let Some(load) = loader else {
        return lexical("this build has no language-model support".to_owned());
    };
    let dir = models.join(MODEL_NAME);
    let missing = missing_files(&dir);
    if !missing.is_empty() {
        return lexical(format!(
            "model not installed ({}): {}",
            missing.join(", "),
            install_hint(models)
        ));
    }
    match load(&dir) {
        Ok(runtime) => (runtime, None),
        Err(e) => lexical(e.to_string()),
    }
}
