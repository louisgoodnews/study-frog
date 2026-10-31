//! Markdown links that point at a PDF of the vault. Pure.

/// A link to a PDF of the vault, with an optional `#page=N`: the path and the 0-based page.
pub fn pdf_target(url: &str) -> Option<(String, usize)> {
    if url.contains("://") || url.starts_with("mailto:") {
        return None;
    }
    let (path, page) = studyfrog_store::pdf_notes::split_source(url);
    path.to_lowercase()
        .ends_with(".pdf")
        .then(|| (path.to_owned(), page.unwrap_or(0)))
}
