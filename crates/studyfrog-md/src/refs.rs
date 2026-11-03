//! `referenced_by`: the other stacks that reference an item, by stack id (ADR 0011). The key is kept like the other
//! untyped keys, as a flow list (`referenced_by: [01HZ..., 01HY...]`), so vaults without it read exactly as before.

use crate::doc::Doc;
use crate::extra::{extra_raw, with_extra};

/// The key in the front matter.
pub const KEY: &str = "referenced_by";

/// The ids written in the key, in order, as written (not checked, duplicates kept). Reads the flow form
/// `[A, B]`; a single bare id is read as a list of one.
pub fn references_of(doc: &Doc) -> Vec<String> {
    let Some(raw) = extra_raw(doc, KEY) else {
        return Vec::new();
    };
    let inner = raw.trim().trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|id| id.trim().trim_matches(|c| c == '"' || c == '\'').trim())
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The document with `referenced_by` set to `ids`, or removed when there are none.
pub fn with_references(doc: &Doc, ids: &[String]) -> Doc {
    if ids.is_empty() {
        return with_extra(doc, KEY, None);
    }
    with_extra(doc, KEY, Some(&format!("[{}]", ids.join(", "))))
}
