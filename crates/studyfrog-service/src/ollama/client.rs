//! The local Ollama service: whether it is missing, not running, running with no models, or ready,
//! with its version and the models it has. Only the loopback address is contacted.

use serde_json::Value;
use studyfrog_agent::provider::HttpGet;

/// The default address of the local Ollama service.
pub const DEFAULT_ADDRESS: &str = "http://127.0.0.1:11434";
/// Seconds to wait for the version.
const VERSION_SECS: u32 = 3;
/// Seconds to wait for the model list.
const TAGS_SECS: u32 = 5;

/// The state of the Ollama service on this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaState {
    /// The `ollama` program is not installed.
    Missing,
    /// It is installed, but the service does not answer.
    NotRunning,
    /// The service answers, and has no models.
    NoModels,
    /// The service answers and has models.
    Ready,
}

/// A model the service has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The tag, such as `qwen2.5:1.5b`.
    pub name: String,
    /// Size on disk in bytes, as the service reports it.
    pub size_bytes: Option<u64>,
    /// Parameter count as the service reports it.
    pub parameter_size: Option<String>,
    /// Quantization as the service reports it.
    pub quantization: Option<String>,
}

/// What the probe found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OllamaStatus {
    /// The state.
    pub state: OllamaState,
    /// The address that was checked.
    pub address: String,
    /// The service's version, when it answered.
    pub version: Option<String>,
    /// The models it has.
    pub installed: Vec<Installed>,
    /// A note for the user (for example why the model list could not be read).
    pub note: Option<String>,
}

/// True when `address` is `http://` or `https://` to this computer only.
pub fn is_loopback(address: &str) -> bool {
    let rest = address
        .strip_prefix("http://")
        .or_else(|| address.strip_prefix("https://"))
        .unwrap_or("");
    let host = rest.split(['/', ':']).next().unwrap_or("");
    let host = if rest.starts_with('[') {
        rest.split(']').next().unwrap_or("").trim_start_matches('[')
    } else {
        host
    };
    matches!(host, "127.0.0.1" | "localhost" | "[::1]" | "::1")
}

/// The version from `/api/version`.
pub fn parse_version(body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(body).ok()?;
    v.get("version")?.as_str().map(str::to_owned)
}

/// The models from `/api/tags`.
pub fn parse_tags(body: &str) -> Result<Vec<Installed>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| format!("not JSON: {e}"))?;
    let list = v
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| "no model list in the answer".to_owned())?;
    Ok(list.iter().filter_map(installed_of).collect())
}

fn installed_of(m: &Value) -> Option<Installed> {
    let name = m.get("name")?.as_str()?.to_owned();
    let details = m.get("details");
    let text = |key: &str| {
        details
            .and_then(|d| d.get(key))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    Some(Installed {
        name,
        size_bytes: m.get("size").and_then(Value::as_u64),
        parameter_size: text("parameter_size"),
        quantization: text("quantization_level"),
    })
}

/// The state from what the probe found. `version` is `None` when the service did not answer.
pub fn status_of(
    address: &str,
    version: Option<String>,
    tags: Option<Result<Vec<Installed>, String>>,
    binary_found: bool,
) -> OllamaStatus {
    let base = |state| OllamaStatus {
        state,
        address: address.to_owned(),
        version: None,
        installed: Vec::new(),
        note: None,
    };
    if version.is_none() {
        let state = if binary_found {
            OllamaState::NotRunning
        } else {
            OllamaState::Missing
        };
        return base(state);
    }
    match tags {
        Some(Ok(list)) if !list.is_empty() => OllamaStatus {
            version,
            installed: list,
            ..base(OllamaState::Ready)
        },
        Some(Err(why)) => OllamaStatus {
            version,
            note: Some(format!("The model list could not be read: {why}")),
            ..base(OllamaState::NoModels)
        },
        _ => OllamaStatus {
            version,
            ..base(OllamaState::NoModels)
        },
    }
}

/// Probe the service at `address` with the host's GET. `binary_found` says whether the `ollama`
/// program is installed (see `run::run_timeout`).
pub fn probe(get: HttpGet, address: &str, binary_found: bool) -> OllamaStatus {
    if !is_loopback(address) {
        return OllamaStatus {
            note: Some("Only a service on this computer is checked.".to_owned()),
            ..status_of(address, None, None, binary_found)
        };
    }
    let version = get(&format!("{address}/api/version"), VERSION_SECS)
        .ok()
        .filter(|r| r.status == 200)
        .and_then(|r| parse_version(&r.body));
    let tags = version.as_ref().map(|_| {
        get(&format!("{address}/api/tags"), TAGS_SECS)
            .map_err(|e| e.to_string())
            .and_then(|r| parse_tags(&r.body))
    });
    status_of(address, version, tags, binary_found)
}
