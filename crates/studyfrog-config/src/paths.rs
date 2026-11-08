//! Where StudyFrog keeps its own files: one directory under the user's data home. Pure; no file access.

use std::path::{Path, PathBuf};

/// Names the application home in place of the default one (for isolated tests and self-hosting).
pub const HOME_ENV: &str = "STUDYFROG_HOME";
/// The human-readable application log inside `logs/`.
pub const LOG_TEXT_FILE: &str = "studyfrog.log";
/// The structured application log (JSON Lines) inside `logs/`.
pub const LOG_JSON_FILE: &str = "console.jsonl";

/// The directories inside the application home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// The application home.
    pub root: PathBuf,
    /// `app.toml`, `secrets.toml`, `providers.toml` and `profiles.toml`.
    pub config: PathBuf,
    /// Reserved for per-profile data.
    pub profiles: PathBuf,
    /// The application logs.
    pub logs: PathBuf,
    /// Downloaded language models.
    pub models: PathBuf,
    /// Runtime state of a running instance (sockets, locks).
    pub run: PathBuf,
    /// User scripts, `scripts.toml` and the Python client (`lib/`).
    pub scripts: PathBuf,
}

/// `$STUDYFROG_HOME`, else `$XDG_DATA_HOME/studyfrog`, else `$HOME/.local/share/studyfrog`. Pure:
/// the caller passes the environment values (empty values count as unset).
pub fn app_home(
    studyfrog_home: Option<&str>,
    xdg_data_home: Option<&str>,
    home: Option<&str>,
) -> Option<PathBuf> {
    let non_empty = |v: Option<&str>| v.filter(|s| !s.is_empty()).map(PathBuf::from);
    non_empty(studyfrog_home)
        .or_else(|| non_empty(xdg_data_home).map(|d| d.join("studyfrog")))
        .or_else(|| non_empty(home).map(|h| h.join(".local").join("share").join("studyfrog")))
}

/// The directories inside the application home `root`.
pub fn app_paths(root: &Path) -> AppPaths {
    AppPaths {
        root: root.to_path_buf(),
        config: root.join("config"),
        profiles: root.join("profiles"),
        logs: root.join("logs"),
        models: root.join("models"),
        run: root.join("run"),
        scripts: root.join("scripts"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_override_wins_then_the_data_home_then_the_home_directory() {
        let path = |a, b, c| app_home(a, b, c).unwrap();
        assert_eq!(path(Some("/t"), Some("/x"), Some("/h")), Path::new("/t"));
        assert_eq!(
            path(None, Some("/x"), Some("/h")),
            Path::new("/x/studyfrog")
        );
        assert_eq!(
            path(Some(""), Some(""), Some("/h")),
            Path::new("/h/.local/share/studyfrog")
        );
        assert_eq!(app_home(None, None, None), None);
    }

    #[test]
    fn every_directory_sits_inside_the_home() {
        let p = app_paths(Path::new("/s"));
        assert_eq!(p.config, Path::new("/s/config"));
        assert_eq!(p.logs, Path::new("/s/logs"));
        assert_eq!(p.models, Path::new("/s/models"));
        assert_eq!(p.run, Path::new("/s/run"));
    }
}
