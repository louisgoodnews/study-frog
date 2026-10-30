//! Reading and parsing vault files. A bad file never aborts a scan.

use std::fs;

use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::error::{ParseError, ParseErrorKind};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_md::validate::find_duplicate_ids;

use crate::error::StoreError;
use crate::hash::hash_bytes;
use crate::stackmap::{owning_stack, stack_folders};
use crate::vault::{VaultFile, list_markdown};

/// One file after reading: its hash and either a parsed document or the reason it failed.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannedFile {
    /// Stat data.
    pub file: VaultFile,
    /// Content hash (0 when the file could not be read).
    pub hash: u64,
    /// Owning stack folder, `None` for the inbox.
    pub stack: Option<String>,
    /// Parsed document or error.
    pub outcome: Result<Doc, ParseError>,
}

/// Result of scanning a whole vault.
#[derive(Debug, Clone, PartialEq)]
pub struct Scan {
    /// Every Markdown file, sorted by relative path.
    pub files: Vec<ScannedFile>,
}

/// The error recorded for a file that could not be read.
pub fn read_error(reason: String) -> ParseError {
    ParseError {
        line: None,
        kind: ParseErrorKind::InvalidYaml(reason),
    }
}

/// Read a file and hash it. Read failures become a per-file error message.
pub fn read_file(file: &VaultFile) -> Result<(Vec<u8>, u64), String> {
    let bytes = fs::read(&file.abs).map_err(|e| format!("cannot read file: {e}"))?;
    let hash = hash_bytes(&bytes);
    Ok((bytes, hash))
}

/// Parse file bytes; invalid UTF-8 is a per-file error.
pub fn parse_bytes(bytes: &[u8]) -> Result<Doc, ParseError> {
    match std::str::from_utf8(bytes) {
        Ok(text) => parse_doc(text),
        Err(_) => Err(read_error("file is not valid UTF-8".to_owned())),
    }
}

/// Read, hash and parse a file.
pub fn read_and_parse(file: &VaultFile) -> (u64, Result<Doc, ParseError>) {
    match read_file(file) {
        Ok((bytes, hash)) => (hash, parse_bytes(&bytes)),
        Err(reason) => (0, Err(read_error(reason))),
    }
}

/// Replace the outcome of every file whose uid is shared with another file by an error.
pub fn mark_duplicates(files: &mut [ScannedFile]) {
    let ids: Vec<(String, _)> = files
        .iter()
        .filter_map(|f| {
            f.outcome
                .as_ref()
                .ok()
                .map(|d| (f.file.rel.clone(), doc_meta(d).uid))
        })
        .collect();
    for (uid, paths) in find_duplicate_ids(&ids) {
        let names: Vec<&str> = paths.iter().map(|p| p.as_str()).collect();
        let reason = format!(
            "id {} is used by several files: {}",
            format_ulid(uid),
            names.join(", ")
        );
        for f in files
            .iter_mut()
            .filter(|f| names.contains(&f.file.rel.as_str()))
        {
            f.outcome = Err(duplicate_error(reason.clone()));
        }
    }
}

/// The error recorded for a file whose id clashes with another file.
pub fn duplicate_error(reason: String) -> ParseError {
    ParseError {
        line: None,
        kind: ParseErrorKind::InvalidValue {
            key: "id".to_owned(),
            reason,
        },
    }
}

/// Scan a vault: list, read, parse, assign stacks, flag duplicate ids.
pub fn scan_vault(root: &std::path::Path) -> Result<Scan, StoreError> {
    let listed = list_markdown(root)?;
    let folders = stack_folders(&listed);
    let mut files: Vec<ScannedFile> = listed
        .into_iter()
        .map(|file| {
            let (hash, outcome) = read_and_parse(&file);
            let stack = owning_stack(&file.rel, &folders);
            ScannedFile {
                file,
                hash,
                stack,
                outcome,
            }
        })
        .collect();
    mark_duplicates(&mut files);
    Ok(Scan { files })
}
