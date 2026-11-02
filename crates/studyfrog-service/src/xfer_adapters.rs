//! The format adapters: a file or folder is read into a bundle by the adapter for its kind. A new format is one more record here
//! (see `docs/EXTENDING.md`); nothing else in the app changes.

use std::fs;
use std::path::Path;

use crate::xfer_model::Bundle;
use crate::xfer_zip::{MAX_ENTRY, read_zip};

/// A format the app can read.
#[derive(Debug, Clone, Copy)]
pub struct Adapter {
    /// The name shown in the preview.
    pub name: &'static str,
    /// The file extensions it reads, lower case.
    pub extensions: &'static [&'static str],
    /// Bytes to a bundle, or why the file cannot be read.
    pub read: fn(&[u8]) -> Result<Bundle, String>,
}

fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|_| "the file is not valid UTF-8 text".to_owned())
}

fn json(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_json::from_json(text(bytes)?)
}

fn csv(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_csv::from_csv(text(bytes)?)
}

fn anki(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_anki::from_apkg(bytes)
}

/// A zip is a StudyFrog JSON export (it holds `export.json`) or else an Obsidian vault of Markdown files.
fn zip(bytes: &[u8]) -> Result<Bundle, String> {
    let files = read_zip(bytes)?;
    let Some((_, doc)) = files.iter().find(|(p, _)| p == "export.json") else {
        return crate::xfer_obsidian::from_entries(&files);
    };
    let mut b = json(doc)?;
    b.files = files
        .into_iter()
        .filter_map(|(p, d)| p.strip_prefix("files/").map(|r| (r.to_owned(), d)))
        .collect();
    Ok(b)
}

/// Every adapter, in the order the file dialog lists them.
pub const ADAPTERS: [Adapter; 4] = [
    Adapter {
        name: "StudyFrog export (JSON)",
        extensions: &["json"],
        read: json,
    },
    Adapter {
        name: "StudyFrog export or Obsidian vault (zip)",
        extensions: &["zip"],
        read: zip,
    },
    Adapter {
        name: "CSV or TSV cards",
        extensions: &["csv", "tsv", "txt"],
        read: csv,
    },
    Adapter {
        name: "Anki package",
        extensions: &["apkg"],
        read: anki,
    },
];

/// The adapter for a file name.
pub fn adapter_for(name: &str) -> Option<&'static Adapter> {
    let ext = name.rsplit_once('.')?.1.to_lowercase();
    ADAPTERS
        .iter()
        .find(|a| a.extensions.contains(&ext.as_str()))
}

/// Read a file or an Obsidian vault folder, and say which adapter read it.
pub fn read_source(path: &Path) -> Result<(Bundle, &'static str), String> {
    if path.is_dir() {
        return crate::xfer_obsidian::from_folder(path).map(|b| (b, "Obsidian vault (folder)"));
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let adapter = adapter_for(&name).ok_or_else(|| format!("{name}: no importer reads this kind of file; supported: json, zip, csv, tsv, apkg or an Obsidian vault folder"))?;
    let meta = fs::metadata(path).map_err(|e| format!("{name}: {e}"))?;
    if meta.len() > MAX_ENTRY * 4 {
        return Err(format!("{name} is too large to import"));
    }
    let bytes = fs::read(path).map_err(|e| format!("{name}: {e}"))?;
    (adapter.read)(&bytes).map(|b| (b, adapter.name))
}
