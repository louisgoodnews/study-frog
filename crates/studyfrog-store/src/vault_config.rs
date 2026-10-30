//! Reading `studyfrog.toml` from the vault root.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use studyfrog_config::vault::{VAULT_CONFIG_FILE, parse_vault_settings};
use studyfrog_core::settings::{VaultSettings, vault_settings_default};

use crate::error::{StoreError, io_err};

/// The vault settings. A missing file means the defaults; an unreadable or invalid one is an error
/// naming the file (callers may fall back to the defaults and show the message).
pub fn read_vault_settings(root: &Path) -> Result<VaultSettings, StoreError> {
    let path = root.join(VAULT_CONFIG_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => parse_vault_settings(&text).map_err(|e| StoreError::Config {
            path: VAULT_CONFIG_FILE.to_owned(),
            message: e.to_string(),
        }),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(vault_settings_default()),
        Err(e) => Err(io_err(&path, &e)),
    }
}
