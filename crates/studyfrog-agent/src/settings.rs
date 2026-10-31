//! `providers.toml` (the models the user connected) and `secrets.toml` (their API keys), both in the
//! config folder of the profile in use. Keys never go in the vault.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::provider::{Protocol, Provider, protocol_from_name, protocol_name};

/// File name of the providers.
pub const PROVIDERS_FILE: &str = "providers.toml";
/// File name of the keys.
pub const SECRETS_FILE: &str = "secrets.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProviders {
    #[serde(default)]
    provider: Vec<RawProvider>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProvider {
    name: String,
    protocol: String,
    base_url: String,
    model: String,
    #[serde(default)]
    allowed: bool,
}

/// Why a provider cannot be used, in words; `None` when it is fine.
pub fn provider_problem(p: &Provider, others: &[Provider]) -> Option<String> {
    let line = |s: &str| !s.trim().is_empty() && !s.chars().any(char::is_control);
    if !line(&p.name) || p.name.chars().count() > 60 {
        return Some("The name must be one line of at most 60 characters.".to_owned());
    }
    if others
        .iter()
        .any(|o| o.name.to_lowercase() == p.name.to_lowercase())
    {
        return Some(format!(
            "There is already a connection called \"{}\".",
            p.name
        ));
    }
    if !(p.base_url.starts_with("http://") || p.base_url.starts_with("https://"))
        || !line(&p.base_url)
    {
        return Some("The address must start with http:// or https://.".to_owned());
    }
    (!line(&p.model)).then(|| "Give the model's name.".to_owned())
}

/// Parse `providers.toml`.
pub fn parse_providers(text: &str) -> Result<Vec<Provider>, String> {
    let raw: RawProviders = toml::from_str(text).map_err(|e| e.message().to_owned())?;
    let mut out: Vec<Provider> = Vec::new();
    for r in raw.provider {
        let protocol = protocol_from_name(&r.protocol)
            .ok_or_else(|| format!("unknown protocol \"{}\"", r.protocol))?;
        let p = Provider {
            name: r.name,
            protocol,
            base_url: r.base_url.trim_end_matches('/').to_owned(),
            model: r.model,
            allowed: r.allowed,
        };
        if let Some(why) = provider_problem(&p, &out) {
            return Err(why);
        }
        out.push(p);
    }
    Ok(out)
}

fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The text of `providers.toml`.
pub fn render_providers(list: &[Provider]) -> String {
    let mut out =
        String::from("# Models StudyFrog may talk to. API keys are in secrets.toml, never here.\n");
    for p in list {
        let protocol: Protocol = p.protocol;
        out.push_str(&format!(
            "\n[[provider]]\nname = {}\nprotocol = {}\nbase_url = {}\nmodel = {}\nallowed = {}\n",
            quoted(&p.name),
            quoted(protocol_name(protocol)),
            quoted(&p.base_url),
            quoted(&p.model),
            p.allowed
        ));
    }
    out
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSecrets {
    #[serde(default)]
    keys: BTreeMap<String, String>,
}

/// Parse `secrets.toml`: connection name to API key.
pub fn parse_secrets(text: &str) -> Result<BTreeMap<String, String>, String> {
    toml::from_str::<RawSecrets>(text)
        .map(|r| r.keys)
        .map_err(|e| e.message().to_owned())
}

/// The text of `secrets.toml`.
pub fn render_secrets(keys: &BTreeMap<String, String>) -> String {
    let mut out = String::from(
        "# API keys. Keep this file private; StudyFrog writes it readable by you only.\n[keys]\n",
    );
    for (name, key) in keys {
        out.push_str(&format!("{} = {}\n", quoted(name), quoted(key)));
    }
    out
}
