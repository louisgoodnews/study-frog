//! The user's own help pages in a wiki proposal: what the model is told and how a page change is checked.
//! The model sees only the user's pages (ids start with [`HELP_PREFIX`]); the supplied help pages are
//! never sent, so a change to one can never pass the checks.

use serde_json::Value;

use crate::wiki::{Change, line_of, text_of};

/// Ids of the user's help pages start with this; the rest is the file name without `.md`.
pub const HELP_PREFIX: &str = "help:";

/// The part of the instructions about help pages.
pub const PAGE_RULES: &str = " Items with an id that starts with help: are the user's own help pages about using \
     StudyFrog. You may propose a new help page as {\"action\": \"create_page\", \"name\": \"<file name>\", \
     \"text\": \"markdown starting with # Title\"} or new text for a page you were shown as \
     {\"action\": \"update_page\", \"page\": \"help:<file name>\", \"text\": \"the whole new markdown\"}. \
     Page names are short and contain no / \\ or :.";

/// The id the model sees for the user's page `file`.
pub fn page_id(file: &str) -> String {
    format!("{HELP_PREFIX}{file}")
}

fn name_of(v: &Value, key: &str) -> Option<String> {
    line_of(v, key)
        .map(|n| n.strip_prefix(HELP_PREFIX).unwrap_or(&n).trim().to_owned())
        .filter(|n| !n.is_empty() && !n.starts_with('.'))
        .filter(|n| !n.chars().any(|c| matches!(c, '/' | '\\' | ':')))
        .filter(|n| n.chars().count() <= 80)
}

/// Check a `create_page` or `update_page` change. `uids` are the ids the model was shown.
pub fn page_change(action: &str, v: &Value, uids: &[String]) -> Result<Change, String> {
    let text =
        text_of(v, "text").ok_or_else(|| format!("{action}: \"text\" is missing or not usable"))?;
    if action == "create_page" {
        let name = name_of(v, "name").ok_or("create_page: the name is missing or not usable")?;
        return Ok(Change::CreatePage { name, text });
    }
    let shown = text_of(v, "page")
        .filter(|p| p.starts_with(HELP_PREFIX) && uids.contains(p))
        .ok_or("update_page: only the user's own help pages you were shown can be changed")?;
    let name = shown[HELP_PREFIX.len()..].to_owned();
    Ok(Change::UpdatePage { name, text })
}
