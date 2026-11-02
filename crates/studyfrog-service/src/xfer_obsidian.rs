//! The Obsidian importer: a vault folder (or a zip of one) becomes a bundle. Folders become stacks, Markdown files become notes with
//! their front matter, wikilinks, tags and pictures; syntax StudyFrog does not support stays as text and is counted for the preview.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use studyfrog_core::plan::ItemKind;

use crate::xfer_model::{Bundle, XStack, item, safe_folder};
use crate::xfer_obsidian_front::{count_unsupported, inline_tags, read_front, split_front};
use crate::xfer_plan::MAX_ITEMS;
use crate::xfer_zip::{Entry, MAX_ENTRY, MAX_FILES};

const MEDIA_DIR: &str = "images/obsidian";
const PICTURES: [&str; 5] = ["png", "jpg", "jpeg", "gif", "pdf"];
const FALLBACK_STACK: &str = "Obsidian";

fn extension(name: &str) -> String {
    name.rsplit_once('.')
        .map_or(String::new(), |(_, e)| e.to_lowercase())
}

fn clean_folder(dir: &str) -> String {
    let parts: Vec<String> = dir
        .split('/')
        .map(|p| {
            p.trim()
                .replace('\\', "-")
                .trim_start_matches('.')
                .to_owned()
        })
        .filter(|p| !p.is_empty())
        .collect();
    let folder = if parts.is_empty() {
        FALLBACK_STACK.to_owned()
    } else {
        parts.join("/")
    };
    if safe_folder(&folder) {
        folder
    } else {
        FALLBACK_STACK.to_owned()
    }
}

fn find_file<'a>(files: &'a [Entry], name: &str) -> Option<&'a Entry> {
    let base = name.rsplit('/').next().unwrap_or(name).to_lowercase();
    files.iter().find(|(p, _)| {
        p.rsplit('/')
            .next()
            .is_some_and(|n| n.to_lowercase() == base)
            && PICTURES.contains(&extension(p).as_str())
    })
}

/// Pictures embedded with `![[name.png]]` or linked with `![alt](name.png)` become links into the media folder; the files are collected.
fn pictures(body: &str, files: &[Entry], b: &mut Bundle, embeds: &mut usize) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(i) = rest.find("![") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 2..];
        let (name, len) = if let Some(inner) = after.strip_prefix('[') {
            inner
                .find("]]")
                .map(|e| {
                    (
                        inner[..e].split('|').next().unwrap_or_default().to_owned(),
                        e + 4,
                    )
                })
                .unwrap_or_default()
        } else {
            after
                .find("](")
                .and_then(|m| {
                    after[m + 2..]
                        .find(')')
                        .map(|e| (after[m + 2..m + 2 + e].to_owned(), m + 3 + e))
                })
                .unwrap_or_default()
        };
        let ext = extension(&name);
        if len == 0 || name.starts_with("http") {
            out.push_str("![");
            rest = after;
        } else if PICTURES.contains(&ext.as_str()) {
            if let Some((_, data)) = find_file(files, &name) {
                let target = format!("{MEDIA_DIR}/{}", name.rsplit('/').next().unwrap_or(&name));
                if !b.files.iter().any(|(p, _)| *p == target) {
                    b.files.push((target.clone(), data.clone()));
                }
                out.push_str(&format!("![]({target})"));
            } else {
                out.push_str(&rest[i..i + 2 + len]);
            }
            rest = &after[len..];
        } else {
            *embeds += 1;
            out.push_str(&rest[i..i + 2 + len]);
            rest = &after[len..];
        }
    }
    out + rest
}

/// Read the files of an Obsidian vault: Markdown files and the pictures next to them.
pub fn from_entries(files: &[Entry]) -> Result<Bundle, String> {
    let mut b = Bundle::default();
    let mut totals: BTreeMap<&'static str, usize> = BTreeMap::new();
    let (mut embeds, mut dropped_keys) = (0, Vec::new());
    for (path, bytes) in files {
        let parts: Vec<&str> = path.split('/').collect();
        if extension(path) != "md" || parts.iter().any(|p| p.starts_with('.')) {
            continue;
        }
        if b.items.len() >= MAX_ITEMS {
            return Err(format!(
                "the vault holds more than {MAX_ITEMS} notes; one import is limited to that"
            ));
        }
        let Ok(text) = std::str::from_utf8(bytes) else {
            b.warnings
                .push(format!("{path} is not valid UTF-8 and was left out"));
            continue;
        };
        let (front_lines, body) = split_front(text);
        let front = read_front(&front_lines);
        let folder = clean_folder(&parts[..parts.len() - 1].join("/"));
        if !b.stacks.iter().any(|s| s.folder == folder) {
            b.stacks.push(XStack {
                folder: folder.clone(),
                title: folder.rsplit('/').next().unwrap_or(&folder).to_owned(),
                description: String::new(),
            });
        }
        for (k, n) in count_unsupported(body) {
            *totals.entry(k).or_insert(0) += n;
        }
        let name = parts.last().map_or("", |n| n.trim_end_matches(".md"));
        let mut it = item(
            ItemKind::Note,
            &folder,
            &pictures(body, files, &mut b, &mut embeds),
            "",
        );
        it.title = front.title.unwrap_or_else(|| name.to_owned());
        it.tags = front.tags;
        for t in inline_tags(body) {
            if !it.tags.contains(&t) {
                it.tags.push(t);
            }
        }
        it.extra = front.extra;
        dropped_keys.extend(front.dropped);
        b.items.push(it);
    }
    for (what, n) in totals {
        b.warnings.push(format!(
            "{n} {what} kept as text (StudyFrog does not render them)"
        ));
    }
    if embeds > 0 {
        b.warnings
            .push(format!("{embeds} embedded notes kept as text"));
    }
    dropped_keys.sort();
    dropped_keys.dedup();
    if !dropped_keys.is_empty() {
        b.warnings.push(format!(
            "front-matter keys not kept: {}",
            dropped_keys.join(", ")
        ));
    }
    if b.items.is_empty() {
        return Err("the folder holds no Markdown notes".to_owned());
    }
    Ok(b)
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<Entry>, total: &mut u64) -> Result<(), String> {
    for entry in fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
    {
        let (path, name) = (
            entry.path(),
            entry.file_name().to_string_lossy().into_owned(),
        );
        if name.starts_with('.') || path.is_symlink() {
            continue;
        }
        if path.is_dir() {
            walk(&path, base, out, total)?;
            continue;
        }
        let ext = extension(&name);
        let size = entry.metadata().map_or(0, |m| m.len());
        if (ext == "md" || PICTURES.contains(&ext.as_str())) && size <= MAX_ENTRY {
            *total += size;
            if out.len() >= MAX_FILES * 4 || *total > crate::xfer_zip::MAX_TOTAL {
                return Err("the folder is too large to import".to_owned());
            }
            let rel = path
                .strip_prefix(base)
                .map_or(name, |p| p.to_string_lossy().replace('\\', "/"));
            out.push((rel, fs::read(&path).map_err(|e| e.to_string())?));
        }
    }
    Ok(())
}

/// Read an Obsidian vault folder.
pub fn from_folder(path: &Path) -> Result<Bundle, String> {
    let mut files = Vec::new();
    walk(path, path, &mut files, &mut 0)?;
    from_entries(&files)
}
