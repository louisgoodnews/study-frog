//! Pictures in Markdown: which paths may be shown, and how a picture reaches the vault. Pure.

/// File extensions the app can show and add.
pub const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "gif"];

/// The `file://` address of a vault-relative picture path, or `None` when the path is not a plain
/// relative path to a picture inside the vault (absolute paths, `..`, web addresses, other file types).
pub fn image_uri(vault_root: &str, rel: &str) -> Option<String> {
    let rel = rel.trim();
    let plain = !rel.is_empty()
        && !rel.starts_with('/')
        && !rel.contains(':')
        && !rel.contains('\\')
        && !rel.split('/').any(|part| part == ".." || part == ".");
    let ext = rel.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())?;
    (plain && IMAGE_EXTENSIONS.contains(&ext.as_str()))
        .then(|| format!("file://{}/{rel}", vault_root.trim_end_matches('/')))
}

/// The Markdown for a picture that was added at `rel`, on its own line.
pub fn image_markdown(rel: &str) -> String {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let alt = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    format!("![{alt}]({rel})")
}

/// Add `snippet` to the end of `text` on a line of its own.
pub fn append_line(text: &str, snippet: &str) -> String {
    let lead = if text.is_empty() || text.ends_with("\n\n") {
        ""
    } else if text.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!("{text}{lead}{snippet}\n")
}
