//! The TOML interchange format for StudyFrog exports (TASK-0220).

use crate::xfer_convert::{bundle_from_export_doc, export_doc_from_bundle};
use crate::xfer_doc::ExportDoc;
use crate::xfer_model::Bundle;

/// What a StudyFrog TOML export file looks like.
pub const TOML_LAYOUT: &str = "A TOML file with format = \"studyfrog-export\", version = 1, a [[stacks]] table array and an [[items]] table array. Each item specifies kind, stack, title, text, answer, and optional tags and also_in references. In advanced mode, items include an [items.average] table and [[items.stacks]] and [[items.reviews]] table arrays.";

/// A sample TOML export file.
pub const TOML_EXAMPLE: &str = r#"format = "studyfrog-export"
version = 1
exported = "2026-01-01T00:00:00Z"
scheduling = false

[[stacks]]
folder = "french"
title = "French"
description = ""

[[items]]
kind = "card"
title = ""
stack = "french"
text = "the cat"
answer = "le chat"
tags = ["animals"]
"#;

/// Serialize a `Bundle` into TOML text.
pub fn to_toml(b: &Bundle, exported: &str) -> String {
    let doc = export_doc_from_bundle(b, exported);
    toml::to_string_pretty(&doc).unwrap_or_default()
}

/// Deserialize a `Bundle` from TOML text.
pub fn from_toml(source: &str) -> Result<Bundle, String> {
    let doc: ExportDoc = toml::from_str(source).map_err(|e| format!("not valid TOML: {e}"))?;
    bundle_from_export_doc(doc)
}
