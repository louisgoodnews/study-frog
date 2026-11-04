//! Reading and writing simple one-line front-matter keys that the format does not type (such as
//! `priority`), through the verbatim "unknown key" entries every document keeps.

use crate::doc::{Doc, RawEntry};

fn entries(doc: &Doc) -> &[RawEntry] {
    match doc {
        Doc::Stack(d) => &d.extra,
        Doc::Note(d) => &d.extra,
        Doc::Card(d) => &d.extra,
        Doc::Question(d) => &d.extra,
        Doc::Diary(d) => &d.extra,
    }
}

fn entries_mut(doc: &mut Doc) -> &mut Vec<RawEntry> {
    match doc {
        Doc::Stack(d) => &mut d.extra,
        Doc::Note(d) => &mut d.extra,
        Doc::Card(d) => &mut d.extra,
        Doc::Question(d) => &mut d.extra,
        Doc::Diary(d) => &mut d.extra,
    }
}

/// The value of a one-line key exactly as stored (trimmed, quotes kept); `None` when the key is absent or empty.
pub fn extra_raw(doc: &Doc, key: &str) -> Option<String> {
    let entry = entries(doc).iter().find(|e| e.key == key)?;
    let (_, value) = entry.text.split_once(':')?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// The value of a one-line key, without quotes; `None` when the key is absent or empty.
pub fn extra_value(doc: &Doc, key: &str) -> Option<String> {
    let entry = entries(doc).iter().find(|e| e.key == key)?;
    let (_, value) = entry.text.split_once(':')?;
    let value = value.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    (!value.is_empty()).then(|| value.to_owned())
}

/// The document with the key set to `value` (a plain word, no line breaks), or removed for `None`.
pub fn with_extra(doc: &Doc, key: &str, value: Option<&str>) -> Doc {
    let mut out = doc.clone();
    let list = entries_mut(&mut out);
    list.retain(|e| e.key != key);
    if let Some(v) = value {
        list.push(RawEntry {
            key: key.to_owned(),
            text: format!("{key}: {v}"),
        });
    }
    out
}
