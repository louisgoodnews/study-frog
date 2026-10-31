//! `manifest.toml`, the first file of a backup: what it holds and which program made it.

use serde::Deserialize;

use crate::error::{ConfigError, config_error, from_toml};
use crate::values::toml_string;

/// File name inside the archive.
pub const MANIFEST_FILE: &str = "manifest.toml";
/// The layout of the archive this version writes and reads.
pub const BACKUP_FORMAT: u32 = 1;

/// What a backup holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Layout version.
    pub format: u32,
    /// When it was made, in Unix milliseconds.
    pub created_ms: i64,
    /// The StudyFrog version that wrote it.
    pub app_version: String,
    /// The vault folder's name (for the file name and the restore screen).
    pub vault_name: String,
    /// Files of the vault (not counting the database and the settings).
    pub files: u32,
    /// Their total size in bytes.
    pub bytes: u64,
    /// The app settings are inside.
    pub has_settings: bool,
    /// The API keys are inside.
    pub has_keys: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    format: u32,
    created_ms: i64,
    app_version: String,
    vault_name: String,
    files: u32,
    bytes: u64,
    has_settings: bool,
    has_keys: bool,
}

/// Parse a manifest; an unknown layout version is refused.
pub fn parse_manifest(text: &str) -> Result<Manifest, ConfigError> {
    let raw: Raw = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    if raw.format != BACKUP_FORMAT {
        return Err(config_error(
            None,
            format!(
                "this backup has layout {}, which this version cannot read (it reads {BACKUP_FORMAT})",
                raw.format
            ),
        ));
    }
    Ok(Manifest {
        format: raw.format,
        created_ms: raw.created_ms,
        app_version: raw.app_version,
        vault_name: raw.vault_name,
        files: raw.files,
        bytes: raw.bytes,
        has_settings: raw.has_settings,
        has_keys: raw.has_keys,
    })
}

/// The file text of a manifest.
pub fn render_manifest(m: &Manifest) -> String {
    format!(
        "format = {}\ncreated_ms = {}\napp_version = {}\nvault_name = {}\nfiles = {}\nbytes = {}\nhas_settings = {}\nhas_keys = {}\n",
        m.format,
        m.created_ms,
        toml_string(&m.app_version),
        toml_string(&m.vault_name),
        m.files,
        m.bytes,
        m.has_settings,
        m.has_keys
    )
}
