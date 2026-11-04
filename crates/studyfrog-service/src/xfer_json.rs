//! The JSON interchange format, version 1 (`docs/specs/interchange.md`).

use studyfrog_core::plan::{ItemKind, item_kind_name};

use crate::xfer_convert::{bundle_from_export_doc, export_doc_from_bundle};
use crate::xfer_doc::ExportDoc;
pub use crate::xfer_doc::{FORMAT, VERSION};
use crate::xfer_model::Bundle;

/// The JSON text of a bundle.
pub fn to_json(b: &Bundle, exported: &str) -> String {
    let doc = export_doc_from_bundle(b, exported);
    serde_json::to_string_pretty(&doc).unwrap_or_default()
}

/// Read a JSON export. Unknown keys are ignored; an unknown version is refused.
pub fn from_json(source: &str) -> Result<Bundle, String> {
    let doc: ExportDoc =
        serde_json::from_str(source).map_err(|e| format!("not valid JSON: {e}"))?;
    bundle_from_export_doc(doc)
}

/// The kind of an item as the CSV and the preview name it.
pub fn kind_word(k: ItemKind) -> &'static str {
    item_kind_name(k)
}
