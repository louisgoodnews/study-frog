//! The per-user config directory and `app.toml` inside it.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use studyfrog_config::app::{APP_CONFIG_FILE, parse_app_settings, render_app_settings};
use studyfrog_core::settings::AppSettings;

use crate::error::{StoreError, io_err};

/// `$XDG_CONFIG_HOME/studyfrog`, else `$HOME/.config/studyfrog`. Pure: the caller passes the
/// environment values (empty values count as unset, as the XDG spec says).
pub fn app_config_dir(xdg_config_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    let non_empty = |v: Option<&str>| v.filter(|s| !s.is_empty()).map(PathBuf::from);
    let base = non_empty(xdg_config_home).or_else(|| non_empty(home).map(|h| h.join(".config")))?;
    Some(base.join("studyfrog"))
}

/// Read `app.toml` from `dir`; a missing file means the defaults.
pub fn read_app_settings(dir: &Path) -> Result<AppSettings, StoreError> {
    let path = dir.join(APP_CONFIG_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => parse_app_settings(&text).map_err(|e| StoreError::Config {
            path: path.display().to_string(),
            message: e.to_string(),
        }),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(e) => Err(io_err(&path, &e)),
    }
}

/// Write `app.toml` into `dir`, creating it. The write goes through a temporary file so a crash
/// cannot leave a half-written settings file.
pub fn write_app_settings(dir: &Path, settings: &AppSettings) -> Result<(), StoreError> {
    fs::create_dir_all(dir).map_err(|e| io_err(dir, &e))?;
    let (target, temp) = (
        dir.join(APP_CONFIG_FILE),
        dir.join(format!("{APP_CONFIG_FILE}.tmp")),
    );
    fs::write(&temp, render_app_settings(settings)).map_err(|e| io_err(&temp, &e))?;
    fs::rename(&temp, &target).map_err(|e| io_err(&target, &e))
}
