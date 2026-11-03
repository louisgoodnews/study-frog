//! How a PDF is named in lists: by its file name, which tells files apart, with the title stored inside the
//! document added only when it says something. Presentation and word processors fill the title with a
//! placeholder such as "PowerPoint-Präsentation", the same for every file. Pure.

/// The file name of a vault-relative path, with its extension.
pub fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn stem(name: &str) -> &str {
    name.rsplit_once('.').map_or(name, |(s, _)| s)
}

/// Placeholders that programs write as a document's title.
const PLACEHOLDERS: [&str; 10] = [
    "powerpoint-präsentation",
    "powerpoint presentation",
    "powerpoint",
    "präsentation",
    "presentation",
    "untitled",
    "unbenannt",
    "dokument",
    "document",
    "folien",
];

/// A title that tells nothing: empty, the file's own name, a program's placeholder (with or without a number) or
/// a name that starts with the program's.
pub fn is_placeholder(title: &str, name: &str) -> bool {
    let t = title.trim().to_lowercase();
    let bare = t.trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ' || c == '-');
    t.is_empty()
        || t == name.to_lowercase()
        || t == stem(name).to_lowercase()
        || PLACEHOLDERS.contains(&bare)
        || t.starts_with("microsoft ")
        || t.starts_with("adobe ")
}

/// The title worth showing next to the file name, if there is one.
pub fn informative_title<'a>(path: &str, title: &'a str) -> Option<&'a str> {
    (!is_placeholder(title, file_name(path))).then_some(title.trim())
}

/// One line for a list or a header: the file name, then the title when it adds something.
pub fn label(path: &str, title: &str) -> String {
    let name = file_name(path);
    match informative_title(path, title) {
        Some(t) => format!("{name} · {t}"),
        None => name.to_owned(),
    }
}
