//! `studyfrog.toml`: vault-wide settings. Everything lives under `[srs]`; every key is optional.

use serde::Deserialize;
use studyfrog_core::scheduler::scheduler_name;
use studyfrog_core::settings::{VaultSettings, vault_settings_default};

use crate::error::{ConfigError, config_error, from_toml, line_of_key};
use crate::values::{float_text, retention_value, scheduler_value};

/// File name of the vault settings, at the vault root.
pub const VAULT_CONFIG_FILE: &str = "studyfrog.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSrs {
    scheduler: Option<String>,
    request_retention: Option<f64>,
    maximum_interval_days: Option<u32>,
    weights: Option<Vec<f64>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVault {
    srs: Option<RawSrs>,
}

fn weights_value(text: &str, w: &[f64]) -> Result<[f32; 21], ConfigError> {
    let bad = |m: &str| config_error(line_of_key(text, "weights"), m.to_owned());
    if w.len() != 21 {
        return Err(bad("weights must list exactly 21 numbers (FSRS-6)"));
    }
    if w.iter().any(|x| !x.is_finite()) {
        return Err(bad("weights must be finite numbers"));
    }
    let mut out = [0.0_f32; 21];
    out.iter_mut().zip(w).for_each(|(o, x)| *o = *x as f32);
    Ok(out)
}

/// Parse and validate `studyfrog.toml`; keys that are absent fall back to the defaults.
pub fn parse_vault_settings(text: &str) -> Result<VaultSettings, ConfigError> {
    let raw: RawVault = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    let srs = raw.srs.unwrap_or(RawSrs {
        scheduler: None,
        request_retention: None,
        maximum_interval_days: None,
        weights: None,
    });
    let base = vault_settings_default();
    let max = srs
        .maximum_interval_days
        .unwrap_or(base.maximum_interval_days);
    if max == 0 {
        return Err(config_error(
            line_of_key(text, "maximum_interval_days"),
            "maximum_interval_days must be at least 1",
        ));
    }
    Ok(VaultSettings {
        scheduler: srs
            .scheduler
            .map(|n| scheduler_value(text, &n, "scheduler"))
            .transpose()?
            .unwrap_or(base.scheduler),
        request_retention: srs
            .request_retention
            .map(|r| retention_value(text, r, "request_retention"))
            .transpose()?
            .unwrap_or(base.request_retention),
        maximum_interval_days: max,
        weights: srs.weights.map(|w| weights_value(text, &w)).transpose()?,
    })
}

/// Render settings as a complete `studyfrog.toml`.
pub fn render_vault_settings(s: &VaultSettings) -> String {
    let mut out = String::from("# StudyFrog vault settings.\n\n[srs]\n");
    out.push_str(&format!(
        "scheduler = \"{}\"\n",
        scheduler_name(s.scheduler)
    ));
    out.push_str(&format!(
        "request_retention = {}\n",
        float_text(s.request_retention)
    ));
    out.push_str(&format!(
        "maximum_interval_days = {}\n",
        s.maximum_interval_days
    ));
    if let Some(w) = s.weights {
        let items: Vec<String> = w.iter().map(|x| float_text(*x)).collect();
        out.push_str(&format!("weights = [{}]\n", items.join(", ")));
    }
    out
}
