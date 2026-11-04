//! Path validation, hostile path rejection, and normalization for folder and file operations.

use std::path::{Path, PathBuf};

use crate::session::ServiceError;

pub(crate) fn invalid(msg: impl Into<String>) -> ServiceError {
    ServiceError::Invalid(msg.into())
}

const RESERVED_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

const FORBIDDEN_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];

/// Returns true if a path segment is hostile, empty, navigation (`.`/`..`), or reserved.
pub fn is_hostile_segment(seg: &str) -> bool {
    let seg = seg.trim();
    if seg.is_empty() || seg == "." || seg == ".." {
        return true;
    }
    if seg.ends_with('.') || seg.ends_with(' ') {
        return true;
    }
    if seg
        .chars()
        .any(|c| c.is_control() || FORBIDDEN_CHARS.contains(&c) || c == '\\' || c == '/')
    {
        return true;
    }
    let stem = seg.split('.').next().unwrap_or(seg).to_ascii_lowercase();
    RESERVED_NAMES.contains(&stem.as_str())
}

/// Validate a relative vault path: must not be empty, absolute, contain drive letters,
/// escape root (`..`), use forbidden chars or device names, or access `.studyfrog`.
pub fn sanitize_rel_path(path: &str) -> Result<String, ServiceError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(invalid("path cannot be empty"));
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return Err(invalid("absolute paths are not allowed"));
    }
    if trimmed.contains(':') {
        return Err(invalid("drive letters or URL schemes are not allowed"));
    }
    let mut parts = Vec::new();
    for seg in trimmed.split('/') {
        if is_hostile_segment(seg) {
            return Err(invalid(format!(
                "invalid or hostile path segment: \"{seg}\""
            )));
        }
        parts.push(seg);
    }
    let rel = parts.join("/");
    if rel == ".studyfrog" || rel.starts_with(".studyfrog/") {
        return Err(invalid("paths inside .studyfrog are reserved"));
    }
    Ok(rel)
}

/// Validate a single file or folder name for rename: cannot contain slashes or hostile chars.
pub fn sanitize_new_name(name: &str) -> Result<String, ServiceError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(invalid("name cannot be empty"));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(invalid("new name cannot contain path separators"));
    }
    if is_hostile_segment(trimmed) {
        return Err(invalid(format!("invalid or hostile name: \"{trimmed}\"")));
    }
    if trimmed == ".studyfrog" {
        return Err(invalid(".studyfrog is reserved"));
    }
    Ok(trimmed.to_owned())
}

/// Validate that a relative path stays inside the vault root and exists/can exist.
pub fn check_vault_rel(root: &Path, rel: &str) -> Result<PathBuf, ServiceError> {
    let clean = sanitize_rel_path(rel)?;
    let full = root.join(&clean);
    if !full.starts_with(root) {
        return Err(invalid("path escapes vault root"));
    }
    Ok(full)
}

/// Verify that the given path is not the vault root or the internal `.studyfrog` store.
pub fn check_not_root_or_essential(rel: &str) -> Result<(), ServiceError> {
    let trimmed = rel.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == "/" {
        return Err(invalid("vault root cannot be modified or deleted directly"));
    }
    if trimmed == ".studyfrog" || trimmed.starts_with(".studyfrog/") {
        return Err(invalid(".studyfrog is reserved for internal index storage"));
    }
    Ok(())
}

/// Check if `child` is equal to `parent` or sits inside `parent`.
pub fn is_subpath_or_equal(parent: &str, child: &str) -> bool {
    let p = parent.trim_matches('/');
    let c = child.trim_matches('/');
    if p.is_empty() {
        return true;
    }
    c == p || c.starts_with(&format!("{p}/"))
}

/// Resolve and normalize relative paths (handling `.` and `..`), ignoring web URLs.
pub fn normalize_segments(base_dir: &str, raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.starts_with("http://")
        || raw.starts_with("https://")
        || raw.starts_with("mailto:")
        || raw.starts_with("data:")
    {
        return None;
    }
    let combined = if raw.starts_with('/') {
        raw.trim_start_matches('/').to_owned()
    } else if base_dir.is_empty() {
        raw.to_owned()
    } else {
        format!("{base_dir}/{raw}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for seg in combined.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        } else if seg == ".." {
            if parts.is_empty() {
                return None;
            }
            parts.pop();
        } else {
            parts.push(seg);
        }
    }
    Some(parts.join("/"))
}
