//! `filters.toml` (vault root, optional): the library filters the user saved, each a name and a query.
//! The query text is not checked here; the library parses it when the filter is opened.

use std::collections::HashSet;

use serde::Deserialize;

use crate::error::{ConfigError, config_error, from_toml};
use crate::subjects::quoted;

/// File name in the vault root.
pub const FILTERS_FILE: &str = "filters.toml";

/// Longest name of a filter, in characters.
pub const MAX_NAME_CHARS: usize = 80;

/// A saved filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedFilter {
    /// The name shown in the Filters menu, unique without regard to case.
    pub name: String,
    /// The query, in the language of the library.
    pub query: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    filter: Vec<RawFilter>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFilter {
    name: String,
    query: String,
}

/// Why a name cannot be used, in words; `None` when it is fine.
pub fn name_problem(name: &str) -> Option<String> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Some("a filter's name must be one line and not empty".to_owned());
    }
    (name.chars().count() > MAX_NAME_CHARS)
        .then(|| format!("a filter's name may have at most {MAX_NAME_CHARS} characters"))
}

fn problem(list: &[SavedFilter]) -> Option<String> {
    let mut seen = HashSet::new();
    for f in list {
        if let Some(why) = name_problem(&f.name) {
            return Some(why);
        }
        if !seen.insert(f.name.trim().to_lowercase()) {
            return Some(format!("the filter \"{}\" is listed twice", f.name));
        }
    }
    None
}

/// Parse `filters.toml`. Unknown keys, empty or repeated names are errors.
pub fn parse_filters(text: &str) -> Result<Vec<SavedFilter>, ConfigError> {
    let raw: RawFile = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    let list: Vec<SavedFilter> = raw
        .filter
        .into_iter()
        .map(|f| SavedFilter {
            name: f.name.trim().to_owned(),
            query: f.query,
        })
        .collect();
    match problem(&list) {
        Some(why) => Err(config_error(None, why)),
        None => Ok(list),
    }
}

/// The list with `filter` added, or replacing the one of the same name (without regard to case).
pub fn with_filter(list: &[SavedFilter], filter: SavedFilter) -> Result<Vec<SavedFilter>, String> {
    let filter = SavedFilter {
        name: filter.name.trim().to_owned(),
        ..filter
    };
    if let Some(why) = name_problem(&filter.name) {
        return Err(why);
    }
    let key = filter.name.to_lowercase();
    let mut out: Vec<SavedFilter> = list.to_vec();
    match out.iter().position(|f| f.name.to_lowercase() == key) {
        Some(i) => out[i] = filter,
        None => out.push(filter),
    }
    Ok(out)
}

/// The list without the filter of that name.
pub fn without_filter(list: &[SavedFilter], name: &str) -> Vec<SavedFilter> {
    let key = name.trim().to_lowercase();
    list.iter()
        .filter(|f| f.name.to_lowercase() != key)
        .cloned()
        .collect()
}

/// The file text for a list.
pub fn render_filters(list: &[SavedFilter]) -> String {
    let mut out = String::from("# Saved library filters of this vault (edited in StudyFrog).\n");
    for f in list {
        let (name, query) = (quoted(&f.name), quoted(&f.query));
        out.push_str(&format!("\n[[filter]]\nname = {name}\nquery = {query}\n"));
    }
    out
}
