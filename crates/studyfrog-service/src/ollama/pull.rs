//! Pulling a model into the local Ollama service (STORY-0212): the request, and the reading of its answer. The request
//! is one POST with streaming off, so the answer comes when the download has finished; the transport reads whole
//! answers only. Ollama checks each downloaded layer's digest itself, so no checksum is computed here.

use serde_json::{Value, json};
use studyfrog_agent::provider::{HttpRequest, HttpResponse};

use super::client::DEFAULT_ADDRESS;

/// Seconds to wait for a whole download; a model can be several gigabytes.
pub const PULL_SECS: u32 = 7200;

/// The model library the local service pulls from (Ollama's default registry, shown to the user before the click).
pub const LIBRARY_HOST: &str = "registry.ollama.ai";

/// The pull request for the model `name` on the local service.
pub fn pull_request(name: &str) -> HttpRequest {
    HttpRequest {
        url: format!("{DEFAULT_ADDRESS}/api/pull"),
        headers: vec![("Content-Type".to_owned(), "application/json".to_owned())],
        body: json!({ "model": name, "stream": false }).to_string(),
        timeout_secs: PULL_SECS,
    }
}

/// Read the answer: `Ok` when the service reports the download as finished, else a sentence for the user.
pub fn pull_outcome(answer: &HttpResponse) -> Result<(), String> {
    let body: Option<Value> = serde_json::from_str(&answer.body).ok();
    let field = |key: &str| {
        body.as_ref()
            .and_then(|v| v.get(key))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    if let Some(why) = field("error") {
        return Err(why);
    }
    if answer.status != 200 {
        return Err(format!(
            "the Ollama service answered with status {}",
            answer.status
        ));
    }
    match field("status").as_deref() {
        Some("success") => Ok(()),
        _ => Err("the Ollama service did not report the download as finished".to_owned()),
    }
}
