//! The answers of the local-model setup commands (STORY-0206): the audit, Ollama's status with the
//! setup steps, and the suggestions. They read the panel's last check, which the plan refreshed
//! before the answer is made. Pure.

use serde_json::{Value, json};
use studyfrog_service::ollama::audit_types::{Audit, Os};
use studyfrog_service::ollama::client::{OllamaState, OllamaStatus};
use studyfrog_service::ollama::fit::{Fit, Suggestion, fits};

use crate::agent_types::ollama::Snapshot;
use crate::state::State;

fn snapshot(state: &State) -> Result<&Snapshot, String> {
    state
        .agent
        .ollama
        .snapshot
        .as_deref()
        .ok_or_else(|| "the check did not finish".to_owned())
}

fn os_name(os: Os) -> &'static str {
    match os {
        Os::Linux => "linux",
        Os::MacOs => "macos",
        Os::Windows => "windows",
        Os::Other => "other",
    }
}

fn audit_value(a: &Audit) -> Value {
    json!({
        "os": os_name(a.os),
        "cpu": {
            "model": a.cpu.model, "logical_cores": a.cpu.logical_cores,
            "avx": a.cpu.avx, "avx2": a.cpu.avx2, "fma": a.cpu.fma,
            "f16c": a.cpu.f16c, "neon": a.cpu.neon,
        },
        "memory": {"total_bytes": a.ram_total, "free_bytes": a.ram_free},
        "gpus": a.gpus.iter().map(|g| json!({
            "name": g.name, "vram_total_bytes": g.vram_total,
            "vram_free_bytes": g.vram_free, "shares_memory": g.shared,
        })).collect::<Vec<Value>>(),
        "disk": {"models_dir": a.models_dir, "free_bytes": a.disk_free},
    })
}

fn state_word(s: OllamaState) -> &'static str {
    match s {
        OllamaState::Missing => "missing",
        OllamaState::NotRunning => "not_running",
        OllamaState::NoModels => "no_models",
        OllamaState::Ready => "ready",
    }
}

fn fit_word(f: Fit) -> &'static str {
    match f {
        Fit::Video => "video_memory",
        Fit::Memory => "memory",
        Fit::TooLarge => "too_large",
        Fit::Unknown => "unknown",
    }
}

fn status_value(s: &OllamaStatus) -> Value {
    json!({
        "state": state_word(s.state),
        "address": s.address,
        "version": s.version,
        "installed": s.installed.iter().map(|i| json!({
            "name": i.name, "size_bytes": i.size_bytes,
            "parameter_size": i.parameter_size, "quantization": i.quantization,
        })).collect::<Vec<Value>>(),
        "note": s.note,
    })
}

fn suggestion_value(s: &Suggestion) -> Value {
    json!({
        "rank": s.rank, "name": s.name, "parameters": s.parameters,
        "quantization": s.quantization, "download_bytes": s.download_bytes,
        "memory_bytes": s.memory_bytes, "fit": fit_word(s.fit), "fits": fits(s.fit),
        "installed": s.installed, "reason": s.reason,
    })
}

/// The audit of this computer; unknown values are `null`.
pub fn audit_json(state: &State) -> Result<Value, String> {
    let snap = snapshot(state)?;
    Ok(json!({"audit": audit_value(&snap.audit)}))
}

/// Ollama's state, version, installed models and the setup steps.
pub fn status_json(state: &State) -> Result<Value, String> {
    let snap = snapshot(state)?;
    let steps: Vec<Value> = snap
        .guide
        .iter()
        .map(|s| json!({"title": s.title, "commands": s.commands, "done": s.done, "next": s.next}))
        .collect();
    Ok(json!({"ollama": status_value(&snap.status), "steps": steps}))
}

/// The suggestions for this computer, best first.
pub fn suggest_json(state: &State) -> Result<Value, String> {
    let snap = snapshot(state)?;
    let list: Vec<Value> = snap.suggestions.iter().map(suggestion_value).collect();
    Ok(json!({"suggestions": list, "catalog_error": snap.catalog_error}))
}
