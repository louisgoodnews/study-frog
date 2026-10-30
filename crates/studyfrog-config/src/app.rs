//! `app.toml`: per-user application settings, outside any vault.

use serde::Deserialize;
use studyfrog_core::settings::AppSettings;

use crate::error::{ConfigError, from_toml};
use crate::values::toml_string;

/// File name inside the user's StudyFrog config directory.
pub const APP_CONFIG_FILE: &str = "app.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawApp {
    vault: Option<String>,
    #[serde(default)]
    recent: Vec<String>,
}

/// Parse `app.toml`. Unknown keys are errors.
pub fn parse_app_settings(text: &str) -> Result<AppSettings, ConfigError> {
    let raw: RawApp = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    Ok(AppSettings {
        vault: raw.vault,
        recent: raw.recent,
    })
}

/// Render settings as `app.toml`.
pub fn render_app_settings(s: &AppSettings) -> String {
    let mut out = String::from("# StudyFrog application settings (this machine and user).\n");
    if let Some(v) = &s.vault {
        out.push_str(&format!("vault = {}\n", toml_string(v)));
    }
    if !s.recent.is_empty() {
        let items: Vec<String> = s.recent.iter().map(|p| toml_string(p)).collect();
        out.push_str(&format!("recent = [{}]\n", items.join(", ")));
    }
    out
}
