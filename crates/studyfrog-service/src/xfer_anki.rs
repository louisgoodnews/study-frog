//! The Anki importer: an `.apkg` file (a zip with the collection as an SQLite file and the media) becomes a bundle.
//! Decks become stacks, basic and cloze notes become cards, tags and pictures are kept, and the review log becomes reviews.

use std::collections::HashMap;
use std::fs;

use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::review::rating_from_value;

use crate::xfer_anki_db::rows;
use crate::xfer_anki_text::numbers;
pub use crate::xfer_anki_text::{cloze, html_to_text};
use crate::xfer_model::{Bundle, XReview, XStack, item, safe_folder};
use crate::xfer_plan::MAX_ITEMS;
use crate::xfer_zip::read_zip;

const MEDIA_DIR: &str = "images/anki";

fn json_map(raw: &str) -> HashMap<String, Value> {
    serde_json::from_str::<serde_json::Map<String, Value>>(raw)
        .map(|m| m.into_iter().collect())
        .unwrap_or_default()
}

fn deck_folder(name: &str) -> String {
    let parts: Vec<String> = name
        .split("::")
        .map(|p| {
            p.trim()
                .replace(['/', '\\'], "-")
                .trim_start_matches('.')
                .to_owned()
        })
        .filter(|p| !p.is_empty())
        .collect();
    let folder = parts.join("/");
    if safe_folder(&folder) {
        folder
    } else {
        "Anki".to_owned()
    }
}

fn media_map(files: &[(String, Vec<u8>)], b: &mut Bundle) -> HashMap<String, String> {
    let names = files
        .iter()
        .find(|(p, _)| p == "media")
        .and_then(|(_, d)| serde_json::from_slice::<serde_json::Map<String, Value>>(d).ok())
        .unwrap_or_default();
    let mut map = HashMap::new();
    for (num, name) in names {
        let (Some(name), Some((_, data))) = (name.as_str(), files.iter().find(|(p, _)| *p == num))
        else {
            continue;
        };
        let clean = name.replace(['/', '\\'], "-");
        if clean.starts_with('.') || !clean.contains('.') {
            continue;
        }
        let path = format!("{MEDIA_DIR}/{clean}");
        b.files.push((path.clone(), data.clone()));
        map.insert(name.to_lowercase(), path);
    }
    map
}

fn open_collection(
    files: &[(String, Vec<u8>)],
) -> Result<(Connection, std::path::PathBuf), String> {
    let (_, data) = files.iter().find(|(p, _)| p == "collection.anki2" || p == "collection.anki21").ok_or("this is not an Anki package: it has no collection.anki2 (export from Anki with the legacy format, \"Support older Anki versions\")")?;
    let path = std::env::temp_dir().join(format!(
        "studyfrog-anki-{}-{}.sqlite",
        std::process::id(),
        data.len()
    ));
    fs::write(&path, data).map_err(|e| e.to_string())?;
    let conn = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("the Anki collection cannot be opened: {e}"))?;
    Ok((conn, path))
}

/// Read an `.apkg` file.
pub fn from_apkg(bytes: &[u8]) -> Result<Bundle, String> {
    let files = read_zip(bytes)?;
    let (conn, path) = open_collection(&files)?;
    let result = read_collection(&conn, &files);
    drop(conn);
    let _ = fs::remove_file(path);
    result
}

fn read_collection(conn: &Connection, files: &[(String, Vec<u8>)]) -> Result<Bundle, String> {
    let (models_raw, decks_raw): (String, String) = conn
        .query_row("SELECT models, decks FROM col LIMIT 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|e| format!("the Anki collection is damaged: {e}"))?;
    let (models, decks) = (json_map(&models_raw), json_map(&decks_raw));
    let mut b = Bundle {
        scheduling: true,
        ..Bundle::default()
    };
    let media = media_map(files, &mut b);
    let (notes, deck_of, note_of, log) = rows(conn)?;
    if notes.len() > MAX_ITEMS {
        return Err(format!(
            "the package holds {} notes; one import is limited to {MAX_ITEMS}",
            notes.len()
        ));
    }
    let mut by_note: HashMap<i64, Vec<usize>> = HashMap::new();
    for (nid, mid, tags, flds) in &notes {
        let is_cloze = models
            .get(&mid.to_string())
            .and_then(|m| m.get("type"))
            .and_then(Value::as_i64)
            == Some(1);
        let fields: Vec<String> = flds
            .split('\u{1f}')
            .map(|f| html_to_text(f, &media))
            .collect();
        let deck = deck_of
            .get(nid)
            .and_then(|d| decks.get(&d.to_string()))
            .and_then(|d| d.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("Anki");
        let folder = deck_folder(deck);
        if !b.stacks.iter().any(|s| s.folder == folder) {
            b.stacks.push(XStack {
                folder: folder.clone(),
                title: folder.rsplit('/').next().unwrap_or(&folder).to_owned(),
                description: String::new(),
            });
        }
        let tag_list: Vec<String> = tags.split_whitespace().map(str::to_owned).collect();
        let first = fields.first().cloned().unwrap_or_default();
        let made: Vec<_> = if is_cloze {
            numbers(&first)
                .into_iter()
                .map(|n| {
                    item(
                        ItemKind::Card,
                        &folder,
                        &cloze(&first, Some(n)),
                        &cloze(&first, None),
                    )
                })
                .collect()
        } else {
            vec![item(
                ItemKind::Card,
                &folder,
                &first,
                fields.get(1).map_or("", String::as_str),
            )]
        };
        for mut it in made {
            it.tags = tag_list.clone();
            by_note.entry(*nid).or_default().push(b.items.len());
            b.items.push(it);
        }
    }
    let mut last: HashMap<i64, i64> = HashMap::new();
    for (at, cid, ease) in log {
        let (Some(nid), Some(rating)) = (
            note_of.get(&cid),
            rating_from_value(ease.clamp(0, 255) as u8),
        ) else {
            continue;
        };
        let Some(&first_item) = by_note.get(nid).and_then(|v| v.first()) else {
            continue;
        };
        let days = last
            .insert(cid, at)
            .map_or(0.0, |p| ((at - p) as f32 / 86_400_000.0).floor());
        b.items[first_item].reviews.push(XReview {
            at_ms: at,
            rating,
            elapsed_days: days,
        });
    }
    if b.items.is_empty() {
        b.warnings.push("the package holds no notes".to_owned());
    }
    Ok(b)
}
