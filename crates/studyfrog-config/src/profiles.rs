//! `profiles.toml` in the config folder: which user profiles exist and which one is in use. A profile
//! has its own `app.toml` (recent vaults, reading speed, answer engine) and its own keys. The first
//! profile, "Default", keeps its files directly in the config folder, so an older install works as is.

use serde::Deserialize;

use crate::error::{ConfigError, config_error, from_toml};
use crate::values::toml_string;

/// File name in the config folder.
pub const PROFILES_FILE: &str = "profiles.toml";
/// The profile every install has.
pub const DEFAULT_PROFILE: &str = "Default";
/// Longest profile name.
pub const MAX_NAME: usize = 40;

/// The profiles and the one in use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profiles {
    /// The profile in use.
    pub current: String,
    /// Every profile, "Default" first.
    pub names: Vec<String>,
}

impl Default for Profiles {
    fn default() -> Self {
        profiles_default()
    }
}

/// Only the default profile.
pub fn profiles_default() -> Profiles {
    Profiles {
        current: DEFAULT_PROFILE.to_owned(),
        names: vec![DEFAULT_PROFILE.to_owned()],
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    current: Option<String>,
    #[serde(default)]
    profile: Vec<RawProfile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    name: String,
}

/// Why `name` cannot be a new profile's name; `None` when it is fine.
pub fn name_problem(name: &str, existing: &[String]) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return Some("Give the profile a name.".to_owned());
    }
    if name.chars().count() > MAX_NAME || name.chars().any(char::is_control) {
        return Some(format!(
            "A profile name is one line of at most {MAX_NAME} characters."
        ));
    }
    existing
        .iter()
        .any(|e| e.to_lowercase() == name.to_lowercase())
        .then(|| format!("There is already a profile called \"{name}\"."))
}

/// The folder name of a profile: letters and digits kept, everything else a dash.
pub fn profile_slug(name: &str) -> String {
    let slug: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug.trim_matches('-').to_owned()
}

/// Parse `profiles.toml`; the default profile is always present and first, and `current` must exist.
pub fn parse_profiles(text: &str) -> Result<Profiles, ConfigError> {
    let raw: RawFile = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    let mut names = vec![DEFAULT_PROFILE.to_owned()];
    for p in raw.profile {
        if let Some(why) = name_problem(&p.name, &names) {
            return Err(config_error(None, why));
        }
        names.push(p.name.trim().to_owned());
    }
    let current = raw.current.unwrap_or_else(|| DEFAULT_PROFILE.to_owned());
    if !names.contains(&current) {
        return Err(config_error(
            None,
            format!("the current profile \"{current}\" is not listed"),
        ));
    }
    Ok(Profiles { current, names })
}

/// The file text.
pub fn render_profiles(p: &Profiles) -> String {
    let mut out = format!("current = {}\n", toml_string(&p.current));
    for n in p.names.iter().filter(|n| *n != DEFAULT_PROFILE) {
        out.push_str(&format!("\n[[profile]]\nname = {}\n", toml_string(n)));
    }
    out
}
