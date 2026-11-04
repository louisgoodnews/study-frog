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
    /// What the file has to look like, in plain words (shown in the dialog, the help and the prompts).
    pub layout: &'static str,
    /// A small file that follows the layout, when it is text.
    pub example: Option<Example>,
    /// Bytes to a bundle, or why the file cannot be read.
    pub read: fn(&[u8]) -> Result<Bundle, String>,
}

/// A sample file that an importer reads.
#[derive(Debug, Clone, Copy)]
pub struct Example {
    /// Its file name.
    pub file: &'static str,
    /// Its text.
    pub text: &'static str,
}

/// The rules for an Obsidian vault folder (it has no file extension, so it is not an [`Adapter`]).
pub const OBSIDIAN_LAYOUT: &str = "A folder of Markdown files, or a zip of one. Each folder becomes a stack and each .md file a note, with its front matter, wikilinks, tags and pictures. Callouts, dataview blocks, comments, highlights, block references and note embeds stay as plain text.";

const JSON_LAYOUT: &str = "One JSON object with \"format\": \"studyfrog-export\" and \"version\": 1, a \"stacks\" list (folder, title, description) and an \"items\" list. Each item has \"kind\" (note, card or question), \"stack\" (a folder), \"title\", \"text\" (a card's front), \"answer\" (its back) and \"tags\" (a list). The review history is read only when \"scheduling\" is true.";

const ZIP_LAYOUT: &str = "A zip made by StudyFrog's JSON export (export.json and a files folder), or a zip of an Obsidian vault: Markdown files in folders.";

/// What a CSV or TSV file has to look like.
pub const CSV_LAYOUT: &str = "Text with one card, question or note per row. A first row can name the columns: front (or question, term) and back (or answer, definition) are needed; stack (or deck), kind (card, question or note; empty means card), title and tags (separated by semicolons) are optional. Without a header the first two columns are the front and the back. The delimiter (comma, semicolon, tab or |) and the encoding (UTF-8 or Latin-1) are detected; in the import dialog you can choose which column means what. Cells with the delimiter or line breaks go in double quotes.";

const ANKI_LAYOUT: &str = "An Anki package. In Anki choose File, Export, format Anki Deck Package (.apkg), and tick Support older Anki versions; tick Include scheduling information to bring your reviews. Each deck becomes a stack; basic and cloze notes become cards.";

const JSON_EXAMPLE: &str = "{\n  \"format\": \"studyfrog-export\",\n  \"version\": 1,\n  \"exported\": \"2026-01-01T00:00:00Z\",\n  \"scheduling\": false,\n  \"stacks\": [\n    {\"folder\": \"french\", \"title\": \"French\", \"description\": \"\"}\n  ],\n  \"items\": [\n    {\"kind\": \"card\", \"title\": \"\", \"stack\": \"french\", \"text\": \"the cat\", \"answer\": \"le chat\", \"tags\": [\"animals\"]}\n  ]\n}\n";

const CSV_EXAMPLE: &str = "stack,kind,title,front,back,tags\nfrench,card,,the cat,le chat,animals;nouns\nfrench,question,Plural,\"What is the plural of \"\"cheval\"\"?\",chevaux,\n";

fn text(bytes: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(bytes).map_err(|_| "the file is not valid UTF-8 text".to_owned())
}

fn json(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_json::from_json(text(bytes)?)
}

fn toml(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_toml::from_toml(text(bytes)?)
}

fn ron(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_ron::from_ron(text(bytes)?)
}

fn csv(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_csv_map::read_csv(bytes, &Default::default()).map(|r| r.bundle)
}

fn anki(bytes: &[u8]) -> Result<Bundle, String> {
    crate::xfer_anki::from_apkg(bytes)
}

/// A zip is a StudyFrog export (export.json, export.toml, export.ron) or an Obsidian vault.
fn zip(bytes: &[u8]) -> Result<Bundle, String> {
    let files = read_zip(bytes)?;
    let parsed = if let Some((_, doc)) = files.iter().find(|(p, _)| p == "export.json") {
        Some(json(doc))
    } else if let Some((_, doc)) = files.iter().find(|(p, _)| p == "export.toml") {
        Some(toml(doc))
    } else if let Some((_, doc)) = files.iter().find(|(p, _)| p == "export.ron") {
        Some(ron(doc))
    } else {
        None
    };
    let Some(res) = parsed else {
        return crate::xfer_obsidian::from_entries(&files);
    };
    let mut b = res?;
    b.files = files
        .into_iter()
        .filter_map(|(p, d)| p.strip_prefix("files/").map(|r| (r.to_owned(), d)))
        .collect();
    Ok(b)
}

/// Every adapter, in the order the file dialog lists them.
pub const ADAPTERS: [Adapter; 6] = [
    Adapter {
        name: "StudyFrog export (JSON)",
        extensions: &["json"],
        layout: JSON_LAYOUT,
        example: Some(Example {
            file: "studyfrog-example.json",
            text: JSON_EXAMPLE,
        }),
        read: json,
    },
    Adapter {
        name: "StudyFrog export (TOML)",
        extensions: &["toml"],
        layout: crate::xfer_toml::TOML_LAYOUT,
        example: Some(Example {
            file: "studyfrog-example.toml",
            text: crate::xfer_toml::TOML_EXAMPLE,
        }),
        read: toml,
    },
    Adapter {
        name: "StudyFrog export (RON)",
        extensions: &["ron"],
        layout: crate::xfer_ron::RON_LAYOUT,
        example: Some(Example {
            file: "studyfrog-example.ron",
            text: crate::xfer_ron::RON_EXAMPLE,
        }),
        read: ron,
    },
    Adapter {
        name: "StudyFrog export or Obsidian vault (zip)",
        extensions: &["zip"],
        layout: ZIP_LAYOUT,
        example: None,
        read: zip,
    },
    Adapter {
        name: "CSV or TSV cards",
        extensions: &["csv", "tsv", "txt"],
        layout: CSV_LAYOUT,
        example: Some(Example {
            file: "studyfrog-example.csv",
            text: CSV_EXAMPLE,
        }),
        read: csv,
    },
    Adapter {
        name: "Anki package",
        extensions: &["apkg"],
        layout: ANKI_LAYOUT,
        example: None,
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

/// Whether the file is read as CSV or TSV (by its extension).
pub fn is_csv_path(path: &Path) -> bool {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
    name.and_then(|n| adapter_for(&n))
        .is_some_and(|a| a.name.starts_with("CSV"))
}

/// Read a CSV or TSV file with the user's choices; the result also tells how it was read and every problem.
pub fn read_csv_source(
    path: &Path,
    options: &crate::xfer_csv_map::CsvOptions,
) -> Result<crate::xfer_csv_map::CsvRead, String> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let meta = fs::metadata(path).map_err(|e| format!("{name}: {e}"))?;
    if meta.len() > MAX_ENTRY * 4 {
        return Err(format!("{name} is too large to import"));
    }
    let bytes = fs::read(path).map_err(|e| format!("{name}: {e}"))?;
    crate::xfer_csv_map::read_csv(&bytes, options)
        .map_err(|e| format!("{name}: {e}. {} expects: {CSV_LAYOUT}", "CSV or TSV cards"))
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
    let adapter = adapter_for(&name).ok_or_else(|| format!("{name}: no importer reads this kind of file; supported: json, toml, ron, zip, csv, tsv, apkg or an Obsidian vault folder"))?;
    let meta = fs::metadata(path).map_err(|e| format!("{name}: {e}"))?;
    if meta.len() > MAX_ENTRY * 4 {
        return Err(format!("{name} is too large to import"));
    }
    let bytes = fs::read(path).map_err(|e| format!("{name}: {e}"))?;
    (adapter.read)(&bytes)
        .map(|b| (b, adapter.name))
        .map_err(|e| format!("{name}: {e}. {} expects: {}", adapter.name, adapter.layout))
}
