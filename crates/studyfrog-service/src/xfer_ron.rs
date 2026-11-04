//! The RON interchange format for StudyFrog exports (TASK-0220).

use ron::ser::PrettyConfig;

use crate::xfer_convert::{bundle_from_export_doc, export_doc_from_bundle};
use crate::xfer_doc::ExportDoc;
use crate::xfer_model::Bundle;

/// What a StudyFrog RON export file looks like.
pub const RON_LAYOUT: &str = "A RON (Rusty Object Notation) file with format: \"studyfrog-export\", version: 1, a stacks list and an items list. Each item specifies kind, stack, title, text, answer, and optional tags and also_in references. In advanced mode, items include average, stacks, and reviews.";

/// A sample RON export file.
pub const RON_EXAMPLE: &str = r#"(
    format: "studyfrog-export",
    version: 1,
    exported: "2026-01-01T00:00:00Z",
    scheduling: false,
    stacks: [
        (
            folder: "french",
            title: "French",
            description: "",
        ),
    ],
    items: [
        (
            kind: "card",
            title: "",
            stack: "french",
            text: "the cat",
            answer: "le chat",
            tags: ["animals"],
        ),
    ],
)
"#;

/// Serialize a `Bundle` into RON text.
pub fn to_ron(b: &Bundle, exported: &str) -> String {
    let doc = export_doc_from_bundle(b, exported);
    ron::ser::to_string_pretty(&doc, PrettyConfig::default()).unwrap_or_default()
}

/// Deserialize a `Bundle` from RON text.
pub fn from_ron(source: &str) -> Result<Bundle, String> {
    let doc: ExportDoc = ron::from_str(source).map_err(|e| format!("not valid RON: {e}"))?;
    bundle_from_export_doc(doc)
}
