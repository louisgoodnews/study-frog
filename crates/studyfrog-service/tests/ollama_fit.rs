#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The setup assistant's fit rule, the ranking of the catalog, the catalog file and the setup guide.

use studyfrog_service::ollama::audit_types::{Audit, GpuInfo, Os};
use studyfrog_service::ollama::catalog::{CatalogModel, builtin_catalog, parse_catalog};
use studyfrog_service::ollama::client::{Installed, OllamaState, OllamaStatus, status_of};
use studyfrog_service::ollama::fit::{Fit, fit_of, rank, vram_free_of};
use studyfrog_service::ollama::guide::steps;

const GIB: u64 = 1024 * 1024 * 1024;
fn model(name: &str, params: u32, need: Option<u64>) -> CatalogModel {
    CatalogModel {
        name: name.to_owned(),
        parameters: format!("{params}M"),
        parameters_millions: params,
        quantization: "Q4_K_M".to_owned(),
        download_bytes: need.map(|n| n / 2),
        memory_bytes: need,
    }
}

fn audit(ram_free: Option<u64>, gpus: Vec<GpuInfo>) -> Audit {
    Audit {
        os: Os::Linux,
        ram_free,
        gpus,
        ..Audit::default()
    }
}

#[test]
fn fit_uses_video_memory_when_the_model_fits_there() {
    assert_eq!(
        fit_of(Some(4 * GIB), Some(8 * GIB), Some(2 * GIB)),
        Fit::Video
    );
}

#[test]
fn fit_falls_back_to_free_memory_then_too_large() {
    assert_eq!(
        fit_of(Some(4 * GIB), Some(2 * GIB), Some(12 * GIB)),
        Fit::Memory
    );
    assert_eq!(fit_of(Some(4 * GIB), None, Some(3 * GIB)), Fit::TooLarge);
}

#[test]
fn a_model_with_unknown_size_never_fits() {
    assert_eq!(fit_of(None, Some(100 * GIB), Some(100 * GIB)), Fit::Unknown);
}

#[test]
fn unknown_free_memory_is_unknown_fit() {
    assert_eq!(fit_of(Some(GIB), None, None), Fit::Unknown);
}

#[test]
fn headroom_keeps_a_model_that_would_fill_the_memory_out() {
    // 1000 MiB free: 80 percent is usable, so a 900 MiB model does not fit.
    let free = 1000 * 1024 * 1024;
    assert_eq!(
        fit_of(Some(900 * 1024 * 1024), None, Some(free)),
        Fit::TooLarge
    );
    assert_eq!(
        fit_of(Some(700 * 1024 * 1024), None, Some(free)),
        Fit::Memory
    );
}

#[test]
fn shared_gpu_memory_is_not_video_memory() {
    let gpus = vec![GpuInfo {
        name: "Apple GPU".into(),
        vram_total: None,
        vram_free: None,
        shared: true,
    }];
    assert_eq!(vram_free_of(&gpus), None);
}

#[test]
fn ranking_puts_fitting_models_first_then_installed_then_larger() {
    let catalog = vec![
        model("big", 7000, Some(9 * GIB)),
        model("small", 1000, Some(GIB)),
        model("mid", 3000, Some(3 * GIB)),
        model("unknown", 500, None),
    ];
    let a = audit(Some(16 * GIB), Vec::new());
    let installed = vec![Installed {
        name: "small".into(),
        size_bytes: None,
        parameter_size: None,
        quantization: None,
    }];
    let ranked = rank(&catalog, &a, &installed);
    let names: Vec<&str> = ranked.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["small", "big", "mid", "unknown"]);
    assert_eq!(ranked[0].rank, 1);
    assert!(ranked[0].installed);
    assert_eq!(ranked[3].fit, Fit::Unknown);
    assert!(!studyfrog_service::ollama::fit::fits(ranked[3].fit));
}

#[test]
fn ranking_follows_the_audit_when_memory_is_small() {
    let catalog = vec![
        model("big", 7000, Some(9 * GIB)),
        model("tiny", 500, Some(GIB)),
    ];
    let ranked = rank(&catalog, &audit(Some(2 * GIB), Vec::new()), &[]);
    assert_eq!(ranked[0].name, "tiny");
    assert_eq!(ranked[1].fit, Fit::TooLarge);
    assert!(ranked[1].reason.contains("Needs more memory"));
}

#[test]
fn the_builtin_catalog_parses_and_every_entry_has_sizes() {
    let catalog = builtin_catalog().expect("the shipped catalog parses");
    assert!(catalog.len() >= 8);
    assert!(
        catalog
            .iter()
            .all(|m| m.download_bytes.is_some() && m.memory_bytes.is_some())
    );
}

#[test]
fn a_catalog_entry_without_a_name_is_refused() {
    let bad = "[[model]]\nname = \" \"\nparameters = \"1B\"\nparameters_millions = 1000\nquantization = \"Q4\"\n";
    assert!(parse_catalog(bad).is_err());
    assert!(parse_catalog("[[model]]\nname = \"x\"\nbogus = 1\n").is_err());
}

#[test]
fn status_states_follow_the_probe() {
    let addr = "http://127.0.0.1:11434";
    assert_eq!(
        status_of(addr, None, None, false).state,
        OllamaState::Missing
    );
    assert_eq!(
        status_of(addr, None, None, true).state,
        OllamaState::NotRunning
    );
    let empty = status_of(addr, Some("0.5.4".into()), Some(Ok(Vec::new())), true);
    assert_eq!(empty.state, OllamaState::NoModels);
    assert_eq!(empty.version.as_deref(), Some("0.5.4"));
}

fn status(state: OllamaState) -> OllamaStatus {
    OllamaStatus {
        state,
        address: "http://127.0.0.1:11434".into(),
        version: None,
        installed: Vec::new(),
        note: None,
    }
}

#[test]
fn guide_marks_the_first_step_not_done_as_next() {
    let missing = steps(Os::Linux, &status(OllamaState::Missing), "qwen2.5:1.5b");
    assert_eq!(missing.len(), 4);
    assert!(!missing[0].done);
    assert!(missing[0].next);
    assert_eq!(
        missing[0].commands,
        vec!["curl -fsSL https://ollama.com/install.sh | sh"]
    );

    let no_models = steps(Os::MacOs, &status(OllamaState::NoModels), "qwen2.5:1.5b");
    assert!(no_models[0].done && no_models[1].done && !no_models[2].done);
    assert!(no_models[2].next);
    assert_eq!(no_models[2].commands, vec!["ollama pull qwen2.5:1.5b"]);

    let ready = steps(Os::Windows, &status(OllamaState::Ready), "qwen2.5:1.5b");
    assert!(ready[3].next);
    assert_eq!(ready[0].commands, vec!["winget install --id Ollama.Ollama"]);
    assert_eq!(ready[3].commands, vec!["ollama run qwen2.5:1.5b"]);
}

#[test]
fn the_same_steps_on_every_system() {
    let titles = |os| -> Vec<String> {
        steps(os, &status(OllamaState::NotRunning), "m")
            .into_iter()
            .map(|s| s.title)
            .collect()
    };
    assert_eq!(titles(Os::Linux), titles(Os::Windows));
    assert_eq!(titles(Os::Linux), titles(Os::Other));
}

#[test]
fn an_unknown_system_gets_no_install_command() {
    let s = steps(Os::Other, &status(OllamaState::Missing), "m");
    assert!(s[0].commands.is_empty());
}

#[test]
fn a_fitting_suggestion_is_marked_so() {
    let ranked = rank(
        &[model("x", 1000, Some(GIB))],
        &audit(Some(8 * GIB), Vec::new()),
        &[],
    );
    assert!(studyfrog_service::ollama::fit::fits(ranked[0].fit));
    let _ = Installed {
        name: String::new(),
        size_bytes: None,
        parameter_size: None,
        quantization: None,
    };
}
