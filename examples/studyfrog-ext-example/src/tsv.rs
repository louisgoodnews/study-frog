//! Tab-separated text into cards.

use studyfrog_ext::types::{ImportedCard, Importer};

/// The importer record.
pub fn importer() -> Importer {
    Importer {
        id: "tab-separated",
        name: "Tab-separated text (front, back, tags)",
        extensions: &["tsv", "txt"],
        import: import_tsv,
    }
}

/// One card per line. Lines starting with `#` (such as `#separator:tab`) and empty lines are
/// skipped; a line needs at least a front and a back; a third column holds tags.
pub fn import_tsv(text: &str) -> Result<Vec<ImportedCard>, String> {
    crate::delimited::parse_delimited(text, Some('\t'))
}
