//! Looking for what the user could import: Obsidian vaults (from Obsidian's own list of vaults), Anki packages
//! (`.apkg` files in the usual folders) and Anki profiles. Read-only: a name, a path and a count, nothing more.
//! The directories are passed in, so this stays pure.

use std::fs;
use std::path::{Path, PathBuf};

/// What was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundKind {
    /// A folder of Markdown files Obsidian keeps; it can be imported as it is.
    ObsidianVault,
    /// An `.apkg` file; it can be imported as it is.
    AnkiPackage,
    /// An Anki profile: its data is not read directly, the user exports a package from Anki.
    AnkiProfile,
}

/// One suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// What it is.
    pub kind: FoundKind,
    /// Its name.
    pub name: String,
    /// The file or folder (for a profile: the profile's folder).
    pub path: String,
    /// A count or size in words.
    pub detail: String,
}

/// Most suggestions returned.
pub const MAX_FOUND: usize = 30;
/// Most Markdown files counted in one vault (the count says "more than" beyond it).
pub const COUNT_CAP: usize = 5000;
/// Folders under the home folder searched for `.apkg` files (top level only).
pub const PACKAGE_FOLDERS: [&str; 3] = ["Downloads", "Documents", "Desktop"];

/// Where the directories of the user are.
#[derive(Debug, Clone)]
pub struct Places {
    /// The home folder.
    pub home: PathBuf,
    /// `$XDG_DATA_HOME`, else `~/.local/share`.
    pub data: PathBuf,
    /// `$XDG_CONFIG_HOME`, else `~/.config`.
    pub config: PathBuf,
}

/// The places of the user from the home folder and the two XDG variables (empty or relative ones are ignored).
pub fn places(home: &Path, xdg_data: Option<&str>, xdg_config: Option<&str>) -> Places {
    let pick = |v: Option<&str>, fallback: &str| {
        v.map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join(fallback))
    };
    Places {
        home: home.to_path_buf(),
        data: pick(xdg_data, ".local/share"),
        config: pick(xdg_config, ".config"),
    }
}

fn count_markdown(dir: &Path, n: &mut usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.filter_map(Result::ok) {
        if *n > COUNT_CAP {
            return;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let Ok(kind) = e.file_type() else { continue };
        if name.starts_with('.') || kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            count_markdown(&e.path(), n);
        } else if name.to_lowercase().ends_with(".md") {
            *n += 1;
        }
    }
}

fn vault_paths(config_file: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(config_file) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    v.get("vaults")
        .and_then(|v| v.as_object())
        .into_iter()
        .flatten()
        .filter_map(|(_, vault)| vault.get("path")?.as_str().map(str::to_owned))
        .collect()
}

fn obsidian(p: &Places) -> Vec<Found> {
    let files = [
        p.config.join("obsidian/obsidian.json"),
        p.home
            .join(".var/app/md.obsidian.Obsidian/config/obsidian/obsidian.json"),
        p.home
            .join("snap/obsidian/current/.config/obsidian/obsidian.json"),
    ];
    let mut out: Vec<Found> = Vec::new();
    for path in files.iter().flat_map(|f| vault_paths(f)) {
        let dir = Path::new(&path);
        if !dir.is_absolute() || !dir.is_dir() || out.iter().any(|f| f.path == path) {
            continue;
        }
        let mut n = 0;
        count_markdown(dir, &mut n);
        let detail = if n > COUNT_CAP {
            format!("more than {COUNT_CAP} Markdown files")
        } else {
            format!("{n} Markdown file{}", if n == 1 { "" } else { "s" })
        };
        let name = dir
            .file_name()
            .map_or(path.clone(), |s| s.to_string_lossy().into_owned());
        out.push(Found {
            kind: FoundKind::ObsidianVault,
            name,
            path,
            detail,
        });
    }
    out
}

fn packages(p: &Places) -> Vec<Found> {
    let mut out = Vec::new();
    for folder in PACKAGE_FOLDERS {
        let Ok(entries) = fs::read_dir(p.home.join(folder)) else {
            continue;
        };
        let mut names: Vec<_> = entries.filter_map(Result::ok).collect();
        names.sort_by_key(|e| e.file_name());
        for e in names {
            let name = e.file_name().to_string_lossy().into_owned();
            let is_package = name.to_lowercase().ends_with(".apkg") && !name.starts_with('.');
            let Some(meta) = e.metadata().ok().filter(|m| is_package && m.is_file()) else {
                continue;
            };
            out.push(Found {
                kind: FoundKind::AnkiPackage,
                name,
                path: e.path().display().to_string(),
                detail: format!("{} KB, in {folder}", meta.len().div_ceil(1024)),
            });
        }
    }
    out
}

fn profiles(p: &Places) -> Vec<Found> {
    let roots = [
        p.data.join("Anki2"),
        p.home.join(".var/app/net.ankiweb.Anki/data/Anki2"),
    ];
    let mut out: Vec<Found> = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        let mut dirs: Vec<_> = entries.filter_map(Result::ok).collect();
        dirs.sort_by_key(|e| e.file_name());
        for e in dirs {
            let has = e.path().join("collection.anki2").is_file();
            if has && !out.iter().any(|f| f.path == e.path().display().to_string()) {
                out.push(Found {
                    kind: FoundKind::AnkiProfile,
                    name: e.file_name().to_string_lossy().into_owned(),
                    path: e.path().display().to_string(),
                    detail: "export a deck from Anki as a package (.apkg, with \"Support older Anki versions\"), then pick the file".to_owned(),
                });
            }
        }
    }
    out
}

/// Everything importable that was found: Obsidian vaults, Anki packages, then Anki profiles.
pub fn discover(p: &Places) -> Vec<Found> {
    let mut all = obsidian(p);
    all.extend(packages(p));
    all.extend(profiles(p));
    all.truncate(MAX_FOUND);
    all
}
