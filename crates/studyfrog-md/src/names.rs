//! Lists of names in one-line front-matter keys such as `subjects`.
//!
//! A plain list is comma-separated (`Maths, Physics`). A list with a name that holds a comma, a quote, a
//! backslash, a bracket or blanks at its ends is written as a flow sequence of double-quoted names,
//! `["Prof. Dr. Künkelmann, Harry", "Maths"]`, which is also valid YAML. Plain lists written by earlier
//! versions read exactly as before.

use crate::doc::Doc;
use crate::extra::{extra_raw, with_extra};

fn needs_quotes(name: &str) -> bool {
    name.contains([',', '"', '\\', '[', ']']) || name != name.trim()
}

fn quote(name: &str) -> String {
    let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// The value to store for a list of names: plain when every name allows it, else a quoted flow sequence.
pub fn encode_names(names: &[String]) -> String {
    if names.iter().any(|n| needs_quotes(n)) {
        let items: Vec<String> = names.iter().map(|n| quote(n)).collect();
        return format!("[{}]", items.join(", "));
    }
    names.join(", ")
}

fn plain_names(value: &str) -> Vec<String> {
    value
        .trim_matches(|c| c == '"' || c == '\'')
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

fn push_name(out: &mut Vec<String>, current: &mut String, was_quoted: &mut bool) {
    let name = if *was_quoted {
        current.clone()
    } else {
        current.trim().to_owned()
    };
    if !name.is_empty() {
        out.push(name);
    }
    current.clear();
    *was_quoted = false;
}

fn quoted_names(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let (mut in_quote, mut escaped, mut was_quoted) = (false, false, false);
    for c in inner.chars() {
        match (in_quote, escaped, c) {
            (true, true, c) => {
                current.push(c);
                escaped = false;
            }
            (true, false, '\\') => escaped = true,
            (true, false, '"') => in_quote = false,
            (true, false, c) => current.push(c),
            (false, _, '"') => (in_quote, was_quoted) = (true, true),
            (false, _, ',') => push_name(&mut out, &mut current, &mut was_quoted),
            (false, _, c) if c.is_whitespace() && (current.is_empty() || was_quoted) => {}
            (false, _, c) => current.push(c),
        }
    }
    push_name(&mut out, &mut current, &mut was_quoted);
    out
}

/// The names in a stored value, in order, without blanks and empty entries.
pub fn decode_names(value: &str) -> Vec<String> {
    let value = value.trim();
    match value.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        Some(inner) => quoted_names(inner),
        None => plain_names(value),
    }
}

/// The names under a key of the document; empty when the key is absent.
pub fn extra_names(doc: &Doc, key: &str) -> Vec<String> {
    extra_raw(doc, key).map_or_else(Vec::new, |v| decode_names(&v))
}

/// The document with the key set to the list, or removed when the list is empty.
pub fn with_extra_names(doc: &Doc, key: &str, names: &[String]) -> Doc {
    let value = encode_names(names);
    with_extra(doc, key, Some(value.as_str()).filter(|v| !v.is_empty()))
}
