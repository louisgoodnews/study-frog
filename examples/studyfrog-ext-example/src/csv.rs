//! Comma-separated values into cards.

use studyfrog_ext::types::{ImportedCard, Importer};

/// The importer record.
pub fn importer() -> Importer {
    Importer {
        id: "csv",
        name: "Comma-separated values (front, back, tags)",
        extensions: &["csv"],
        import: import_csv,
    }
}

/// Parse CSV text into cards, supporting RFC 4180 quotes, prompt headers, and auto-detection.
pub fn import_csv(text: &str) -> Result<Vec<ImportedCard>, String> {
    crate::delimited::parse_delimited(text, Some(','))
}
