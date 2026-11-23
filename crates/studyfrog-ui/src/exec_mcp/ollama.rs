//! Performing the setup assistant's effects (STORY-0206): the audit and the Ollama status (read-only,
//! against this computer and the local address only), one short test prompt, and saving the chosen
//! local model in `providers.toml`.

use std::path::PathBuf;

use studyfrog_agent::provider::{Protocol, Provider};
use studyfrog_service::agent::{Ask, load_connections, save_providers, test_connection};
use studyfrog_service::ollama::audit_probe::{Inputs, current_os, read_audit};
use studyfrog_service::ollama::catalog::builtin_catalog;
use studyfrog_service::ollama::client::{DEFAULT_ADDRESS, probe, status_of};
use studyfrog_service::ollama::fit::{fits, rank};
use studyfrog_service::ollama::guide::steps;
use studyfrog_service::ollama::run::{RunError, run_timeout};

use crate::agent_types::ollama::{LOCAL_NAME, OllamaMsg, Snapshot, msg};
use crate::exec::Worker;
use crate::msg::Msg;

/// The model the guide uses when nothing fits.
const FALLBACK_MODEL: &str = "qwen2.5:1.5b";

/// Where Ollama keeps its models: `OLLAMA_MODELS`, else `~/.ollama/models`.
pub fn ollama_models_dir() -> Option<PathBuf> {
    let set = std::env::var_os("OLLAMA_MODELS").filter(|v| !v.is_empty());
    set.map(PathBuf::from).or_else(|| {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
        Some(PathBuf::from(home).join(".ollama").join("models"))
    })
}

/// Whether the `ollama` program is installed (a read-only `--version`).
fn binary_found() -> bool {
    !matches!(
        run_timeout("ollama", &["--version"], 3),
        Err(RunError::NotFound)
    )
}

/// Check this computer and the Ollama service; the answer is the whole snapshot.
pub fn refresh(w: &Worker) -> Msg {
    let audit = read_audit(&Inputs {
        os: current_os(),
        models_dir: ollama_models_dir(),
    });
    let binary = binary_found();
    let status = match w.get {
        Some(get) => probe(get, DEFAULT_ADDRESS, binary),
        None => status_of(DEFAULT_ADDRESS, None, None, binary),
    };
    let (suggestions, catalog_error) = match builtin_catalog() {
        Ok(c) => (rank(&c, &audit, &status.installed), None),
        Err(e) => (Vec::new(), Some(e)),
    };
    let pick = suggestions
        .iter()
        .find(|s| fits(s.fit))
        .or(suggestions.first())
        .map_or_else(|| FALLBACK_MODEL.to_owned(), |s| s.name.clone());
    let guide = steps(audit.os, &status, &pick);
    msg(OllamaMsg::Loaded(Box::new(Snapshot {
        audit,
        status,
        suggestions,
        guide,
        catalog_error,
    })))
}

fn local_provider(model: &str) -> Provider {
    Provider {
        name: LOCAL_NAME.to_owned(),
        protocol: Protocol::Ollama,
        base_url: DEFAULT_ADDRESS.to_owned(),
        model: model.to_owned(),
        allowed: true,
    }
}

/// One short test prompt to the local model, through the same path as every question.
pub fn test_model(w: &Worker, model: &str, now_ns: i64) -> Msg {
    let provider = local_provider(model);
    let result = match (w.transport, w.config_dir.as_deref()) {
        (Some(transport), Some(config_dir)) => {
            let ctx = Ask {
                config_dir,
                log_dir: w.state_dir.as_deref(),
                now_ms: now_ns / 1_000_000,
            };
            test_connection(transport, &provider, ctx)
        }
        (None, _) => Err("This build cannot reach the network.".to_owned()),
        (_, None) => Err("there is no config folder".to_owned()),
    };
    msg(OllamaMsg::Tested(model.to_owned(), result))
}

/// Save the tested model as the local connection (replacing an earlier local choice).
pub fn set_local(w: &Worker, model: &str) -> Msg {
    let Some(dir) = w.config_dir.as_deref() else {
        return msg(OllamaMsg::LocalSet(Err(
            "there is no config folder".to_owned()
        )));
    };
    let mut list: Vec<Provider> = load_connections(dir)
        .providers
        .into_iter()
        .filter(|p| p.name != LOCAL_NAME)
        .collect();
    list.push(local_provider(model));
    let result = save_providers(dir, &list)
        .map(|()| load_connections(dir))
        .map_err(|e| e.to_string());
    msg(OllamaMsg::LocalSet(result))
}
