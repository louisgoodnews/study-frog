//! The format adapters: a file or folder is read into a bundle by the adapter for its kind. A new format is one more record here
//! (see `docs/EXTENDING.md`); nothing else in the app changes.

use std::fs;
use std::path::Path;

use crate::xfer_model::Bundle;
use crate::xfer_zip::{MAX_ENTRY, read_zip};

use crate::xfer_layouts::{
    ANKI_LAYOUT, CSV_EXAMPLE, JSON_EXAMPLE, JSON_LAYOUT, PDF_LAYOUT, ZIP_LAYOUT,
};
pub use crate::xfer_layouts::{CSV_LAYOUT, OBSIDIAN_LAYOUT};

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
pub const ADAPTERS: [Adapter; 8] = [
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
    Adapter {
        name: crate::xfer_pdf::PDF_NAME,
        extensions: &["pdf"],
        layout: PDF_LAYOUT,
        example: None,
        read: crate::xfer_pdf::from_pdf,
    },
    Adapter {
        name: "Markdown cards (one per ## heading) or one note",
        extensions: &["md"],
        layout: crate::xfer_markdown::MARKDOWN_LAYOUT,
        example: Some(Example {
            file: "studyfrog-example.md",
            text: crate::xfer_markdown::MARKDOWN_EXAMPLE,
        }),
        read: crate::xfer_markdown::from_markdown,
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
    let adapter = adapter_for(&name).ok_or_else(|| format!("{name}: no importer reads this kind of file; supported: json, toml, ron, zip, csv, tsv, apkg, pdf, md or an Obsidian vault folder"))?;
    let meta = fs::metadata(path).map_err(|e| format!("{name}: {e}"))?;
    if meta.len() > MAX_ENTRY * 4 {
        return Err(format!("{name} is too large to import"));
    }
    let bytes = fs::read(path).map_err(|e| format!("{name}: {e}"))?;
    let mut bundle = (adapter.read)(&bytes)
        .map_err(|e| format!("{name}: {e}. {} expects: {}", adapter.name, adapter.layout))?;
    if adapter.name == crate::xfer_pdf::PDF_NAME {
        crate::xfer_pdf::name_cards(&mut bundle, &name);
    }
    Ok((bundle, adapter.name))
}

/// What the import dialog says about a chosen path before anything is read: which importer it looks
/// like, or that none reads it yet (PDF documents, for one). Pure: the caller says whether it is a
/// folder.
pub fn describe_input(path: &str, is_dir: bool) -> String {
    let path = path.trim();
    if path.is_empty() {
        return String::new();
    }
    if is_dir {
        return "Looks like an Obsidian vault folder.".to_owned();
    }
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match adapter_for(name) {
        Some(a) => format!("Looks like {}.", a.name),
        None => {
            let known: Vec<&str> = ADAPTERS
                .iter()
                .flat_map(|a| a.extensions.iter().copied())
                .collect();
            format!(
                "{name} is not a file an importer reads yet. Supported: {}, or an Obsidian vault folder.",
                known.join(", ")
            )
        }
    }
}
