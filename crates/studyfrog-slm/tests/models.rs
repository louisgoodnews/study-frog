#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Finding models and falling back to the built-in runtime.

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::models::{MODEL_NAME, install_hint, missing_files, models_dir, open_runtime};
use studyfrog_slm::runtime::{SlmError, SlmRuntime};

fn failing(_: &Path) -> Result<Box<dyn SlmRuntime>, SlmError> {
    Err(SlmError::Failed("not a model".into()))
}

fn working(_: &Path) -> Result<Box<dyn SlmRuntime>, SlmError> {
    Ok(Box::new(Lexical))
}

fn temp(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("studyfrog-models-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_models_directory_follows_xdg_then_home() {
    assert_eq!(
        models_dir(Some("/x"), Some("/h")),
        Some(Path::new("/x/studyfrog/models").to_path_buf())
    );
    assert_eq!(
        models_dir(None, Some("/h")),
        Some(Path::new("/h/.local/share/studyfrog/models").to_path_buf())
    );
    assert_eq!(
        models_dir(Some(""), Some("/h")),
        models_dir(None, Some("/h"))
    );
    assert_eq!(models_dir(None, None), None);
}

#[test]
fn missing_files_are_named_and_the_hint_says_where_to_put_them() {
    let dir = temp("missing");
    assert_eq!(missing_files(&dir).len(), 3);
    fs::write(dir.join("config.json"), "{}").unwrap();
    assert_eq!(missing_files(&dir), ["tokenizer.json", "model.safetensors"]);
    let hint = install_hint(&dir);
    assert!(
        hint.contains(MODEL_NAME) && hint.contains(dir.to_str().unwrap()),
        "{hint}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn without_a_model_the_lexical_runtime_is_used_and_the_reason_is_given() {
    let (rt, why) = open_runtime(None, Some(failing));
    assert!(rt.name().contains("word matching") && why.is_some());
    let (_, why) = open_runtime(Some(Path::new("/x")), None);
    assert!(why.unwrap().contains("no language-model support"));

    let dir = temp("empty");
    let (rt, why) = open_runtime(Some(&dir), Some(failing));
    assert!(rt.name().contains("word matching"));
    let why = why.unwrap();
    assert!(
        why.contains("not installed") && why.contains(MODEL_NAME),
        "{why}"
    );
    assert_eq!(rt.embed(&["a b"]).unwrap().len(), 1, "it still works");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_corrupt_model_falls_back_instead_of_failing_the_run() {
    let dir = temp("corrupt");
    let model = dir.join(MODEL_NAME);
    fs::create_dir_all(&model).unwrap();
    for f in ["config.json", "tokenizer.json", "model.safetensors"] {
        fs::write(model.join(f), "not a model").unwrap();
    }
    let (rt, why) = open_runtime(Some(&dir), Some(failing));
    assert!(rt.name().contains("word matching"));
    assert!(why.is_some(), "the problem is reported");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_installed_model_that_loads_is_used_without_a_reason() {
    let dir = temp("loads");
    let model = dir.join(MODEL_NAME);
    fs::create_dir_all(&model).unwrap();
    for f in ["config.json", "tokenizer.json", "model.safetensors"] {
        fs::write(model.join(f), "x").unwrap();
    }
    let (_, why) = open_runtime(Some(&dir), Some(working));
    assert_eq!(why, None);
    let _ = fs::remove_dir_all(&dir);
}
