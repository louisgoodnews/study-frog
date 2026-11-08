//! The agent's connections and conversations: which models the user connected, where their keys are,
//! and one question to one model. Nothing here sends text unless the provider may receive it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::Path;

use studyfrog_agent::provider::{Message, Provider, Transport, host_of, may_send};
use studyfrog_agent::settings::{
    PROVIDERS_FILE, SECRETS_FILE, parse_providers, parse_secrets, provider_problem,
    render_providers, render_secrets,
};
use studyfrog_agent::wire::{build_request, parse_reply};

use crate::backup::set_private;
use crate::session::ServiceError;

/// The connections of the profile in use.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Connections {
    /// The models the user connected.
    pub providers: Vec<Provider>,
    /// Names of the connections that have an API key (the keys themselves are never passed around).
    pub with_key: BTreeSet<String>,
    /// Why a file was not used, if so.
    pub error: Option<String>,
}

fn io_err(path: &Path, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{}: {e}", path.display()))
}

fn keys_of(dir: &Path) -> Result<BTreeMap<String, String>, String> {
    match fs::read_to_string(dir.join(SECRETS_FILE)) {
        Ok(text) => parse_secrets(&text).map_err(|e| format!("{SECRETS_FILE} is not used: {e}")),
        Err(_) => Ok(BTreeMap::new()),
    }
}

/// Read the connections from the config folder `dir`.
pub fn load_connections(dir: &Path) -> Connections {
    let mut out = Connections::default();
    if let Ok(text) = fs::read_to_string(dir.join(PROVIDERS_FILE)) {
        match parse_providers(&text) {
            Ok(p) => out.providers = p,
            Err(e) => out.error = Some(format!("{PROVIDERS_FILE} is not used: {e}")),
        }
    }
    match keys_of(dir) {
        Ok(keys) => out.with_key = keys.into_keys().collect(),
        Err(e) => out.error = out.error.or(Some(e)),
    }
    out
}

fn write_file(dir: &Path, file: &str, text: &str, private: bool) -> Result<(), ServiceError> {
    fs::create_dir_all(dir).map_err(|e| io_err(dir, &e))?;
    let (target, temp) = (dir.join(file), dir.join(format!("{file}.tmp")));
    fs::write(&temp, text).map_err(|e| io_err(&temp, &e))?;
    if private {
        set_private(&temp)?;
    }
    fs::rename(&temp, &target).map_err(|e| io_err(&target, &e))
}

/// Store the list of connections (checked first).
pub fn save_providers(dir: &Path, list: &[Provider]) -> Result<(), ServiceError> {
    for (i, p) in list.iter().enumerate() {
        if let Some(why) = provider_problem(p, &list[..i]) {
            return Err(ServiceError::Invalid(why));
        }
    }
    write_file(dir, PROVIDERS_FILE, &render_providers(list), false)
}

/// Set (or with `None` remove) the API key of a connection. The file is readable by the owner only.
pub fn set_key(dir: &Path, name: &str, key: Option<&str>) -> Result<(), ServiceError> {
    let mut keys = keys_of(dir).map_err(ServiceError::Invalid)?;
    match key.map(str::trim).filter(|k| !k.is_empty()) {
        Some(k) if k.chars().any(char::is_control) => {
            return Err(ServiceError::Invalid("a key is one line".to_owned()));
        }
        Some(k) => keys.insert(name.to_owned(), k.to_owned()),
        None => keys.remove(name),
    };
    write_file(dir, SECRETS_FILE, &render_secrets(&keys), true)
}

/// One line per call in `agent.log`: when, which connection, which host, the answer's status and how
/// long it took. Never the text sent or received, and never a key.
pub fn log_call(log_dir: Option<&Path>, line: &str) {
    let Some(dir) = log_dir else { return };
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("agent.log"))
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Everything one question needs besides the provider.
#[derive(Debug, Clone, Copy)]
pub struct Ask<'a> {
    /// The config folder (for the key).
    pub config_dir: &'a Path,
    /// Where to log the call, if anywhere.
    pub log_dir: Option<&'a Path>,
    /// The time, in Unix milliseconds, for the log.
    pub now_ms: i64,
}

/// Ask `provider` to continue `messages`. Refused before anything is sent when the provider is remote
/// and the user has not allowed it.
pub fn ask(
    transport: Transport,
    provider: &Provider,
    ctx: Ask<'_>,
    system: &str,
    messages: &[Message],
) -> Result<String, String> {
    if !may_send(provider) {
        return Err(format!(
            "Text from your vault may not be sent to {} until you allow it for that connection.",
            provider.name
        ));
    }
    let key = keys_of(ctx.config_dir)?.remove(&provider.name);
    let request = build_request(provider, key.as_deref(), system, messages)?;
    let host = host_of(&request.url).to_owned();
    let result = transport(&request).and_then(|r| parse_reply(provider, &r).map(|t| (r.status, t)));
    let status = result.as_ref().map_or_else(
        |e| format!("failed: {}", e.chars().take(80).collect::<String>()),
        |(s, _)| s.to_string(),
    );
    log_call(
        ctx.log_dir,
        &format!("{} {} {host} {status}", ctx.now_ms, provider.name),
    );
    result.map(|(_, text)| text)
}

/// Check that a connection answers: one tiny request with no vault text in it, through the same
/// path as every question (the same consent rule for remote providers, the same log line). The
/// answer is a sentence for the user; the API key never appears in it, even when a server echoes it.
pub fn test_connection(
    transport: Transport,
    provider: &Provider,
    ctx: Ask<'_>,
) -> Result<String, String> {
    let ping = [Message {
        role: studyfrog_agent::provider::Role::User,
        content: "Reply with the single word OK.".to_owned(),
    }];
    let key = keys_of(ctx.config_dir)
        .ok()
        .and_then(|mut k| k.remove(&provider.name))
        .filter(|k| !k.is_empty());
    let hide = |text: String| match &key {
        Some(k) => text.replace(k.as_str(), "[API key]"),
        None => text,
    };
    ask(
        transport,
        provider,
        ctx,
        "This is a connection test.",
        &ping,
    )
    .map(|_| {
        format!(
            "{} answered: the connection to {} works with the model {}.",
            provider.name,
            host_of(&provider.base_url),
            provider.model
        )
    })
    .map_err(hide)
}
