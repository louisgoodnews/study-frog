//! Text preparation before embedding: Unicode NFC, trimmed, runs of whitespace collapsed.

use unicode_normalization::UnicodeNormalization;

/// The comparison form of a text. Case is kept; runtimes decide whether they care.
pub fn normalize_text(text: &str) -> String {
    let composed: String = text.nfc().collect();
    composed.split_whitespace().collect::<Vec<_>>().join(" ")
}
