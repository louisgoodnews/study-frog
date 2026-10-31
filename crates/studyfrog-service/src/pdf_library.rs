//! PDF files as part of the vault: adding one to a stack, and renaming one without breaking the notes
//! and links that point at it.

use std::fs;
use std::io::Read;
use std::path::Path;

use studyfrog_pdf::limits::MAX_FILE_BYTES;
use studyfrog_store::stackmap::STACK_FILE;
use studyfrog_store::vault::list_markdown;

use crate::session::{ServiceError, VaultSession};

fn invalid(text: impl Into<String>) -> ServiceError {
    ServiceError::Invalid(text.into())
}

fn io(path: &Path, e: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{}: {e}", path.display()))
}

/// A file name that is safe in a vault: letters, digits, dots, dashes and underscores; spaces become dashes.
fn clean_name(name: &str) -> String {
    let mut out: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '-'
            }
        })
        .collect();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.replace("-.", ".")
        .trim_matches(|c| c == '-' || c == '.')
        .to_owned()
}

fn is_pdf_header(path: &Path) -> Result<bool, ServiceError> {
    let mut head = [0_u8; 1024];
    let n = fs::File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .map_err(|e| io(path, &e))?;
    Ok(head[..n].windows(5).any(|w| w == b"%PDF-"))
}

/// Copy a PDF into the stack at `folder`; returns its vault-relative path. A name that is taken gets a number.
pub fn add_pdf(
    session: &VaultSession,
    source: &Path,
    folder: &str,
) -> Result<String, ServiceError> {
    let dir = session.root.join(folder);
    if folder.is_empty() || !dir.join(STACK_FILE).is_file() {
        return Err(invalid("choose the stack the PDF goes into"));
    }
    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .map(clean_name)
        .unwrap_or_default();
    if !name.to_lowercase().ends_with(".pdf") || name.len() <= 4 {
        return Err(invalid("that file is not a PDF"));
    }
    let len = fs::metadata(source).map_err(|e| io(source, &e))?.len();
    if len > MAX_FILE_BYTES as u64 {
        return Err(invalid(format!(
            "that PDF is larger than {} MB",
            MAX_FILE_BYTES / 1_000_000
        )));
    }
    if !is_pdf_header(source)? {
        return Err(invalid("that file does not look like a PDF"));
    }
    let (stem, ext) = name.split_at(name.len() - 4);
    let mut target = dir.join(&name);
    let mut n = 2;
    while target.exists() {
        target = dir.join(format!("{stem}-{n}{ext}"));
        n += 1;
    }
    fs::copy(source, &target).map_err(|e| io(&target, &e))?;
    let file = target.file_name().and_then(|n| n.to_str()).unwrap_or(&name);
    Ok(format!("{folder}/{file}"))
}

/// Replace `old` by `new` in every Markdown file of the vault that mentions it; returns how many files changed.
fn rewrite_references(session: &VaultSession, old: &str, new: &str) -> Result<usize, ServiceError> {
    let mut changed = 0;
    for file in list_markdown(&session.root).map_err(ServiceError::Store)? {
        let text = fs::read_to_string(&file.abs).map_err(|e| io(&file.abs, &e))?;
        if !text.contains(old) {
            continue;
        }
        let temp = file.abs.with_extension("md.tmp");
        fs::write(&temp, text.replace(old, new))
            .and_then(|()| fs::rename(&temp, &file.abs))
            .map_err(|e| io(&file.abs, &e))?;
        changed += 1;
    }
    Ok(changed)
}

/// Give a PDF a new file name (same folder). Its notes file moves with it and every note, card or link that
/// names the old path is rewritten. Returns the new path and the number of files that were rewritten.
pub fn rename_pdf(
    session: &VaultSession,
    rel: &str,
    new_name: &str,
) -> Result<(String, usize), ServiceError> {
    let clean = !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty());
    let name = new_name.trim();
    let ok_name = !name.is_empty()
        && !name.contains(['/', '\\'])
        && name.to_lowercase().ends_with(".pdf")
        && name.len() > 4;
    if !clean || !rel.to_lowercase().ends_with(".pdf") || !ok_name {
        return Err(invalid("the new name must be a file name ending in .pdf"));
    }
    let folder = rel.rsplit_once('/').map_or("", |(f, _)| f);
    let new_rel = if folder.is_empty() {
        name.to_owned()
    } else {
        format!("{folder}/{name}")
    };
    let (from, to) = (session.root.join(rel), session.root.join(&new_rel));
    if !from.is_file() {
        return Err(invalid("that PDF is not in the vault"));
    }
    if to.exists() {
        return Err(invalid(format!("{new_rel} already exists")));
    }
    fs::rename(&from, &to).map_err(|e| io(&from, &e))?;
    let (notes_from, notes_to) = (
        session.root.join(format!("{rel}.notes.toml")),
        session.root.join(format!("{new_rel}.notes.toml")),
    );
    if notes_from.is_file() {
        fs::rename(&notes_from, &notes_to).map_err(|e| io(&notes_from, &e))?;
    }
    let changed = rewrite_references(session, rel, &new_rel)?;
    Ok((new_rel, changed))
}
