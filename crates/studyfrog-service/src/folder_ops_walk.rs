//! File walking, rewrite collection, and stack boundary helpers for folder operations.

use std::fs;
use std::path::Path;

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::parse::parse_doc;
use studyfrog_md::refs::{references_of, with_references};
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::overview::StackRow;
use studyfrog_store::schedule::{load_mem_state_in, load_sm2_state_in};
use studyfrog_store::stackmap::{STACK_FILE, parent_folder};

use crate::folder_ops_links::rewrite_markdown_links;
use crate::folder_ops_types::{DroppedMemory, LinkRewrite};
use crate::session::{ServiceError, VaultSession};

fn io_err(path: &str, err: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {err}"))
}

/// Recursively list all files relative to `root` under directory `rel_dir`.
pub fn walk_rel_files(root: &Path, rel_dir: &str) -> Result<Vec<String>, ServiceError> {
    let mut files = Vec::new();
    let dir = if rel_dir.is_empty() {
        root.to_path_buf()
    } else {
        root.join(rel_dir)
    };
    walk_dir_inner(root, &dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_dir_inner(root: &Path, dir: &Path, files: &mut Vec<String>) -> Result<(), ServiceError> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(dir).map_err(|e| ServiceError::Io(e.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|e| ServiceError::Io(e.to_string()))?;
        let p = entry.path();
        if p.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') {
                walk_dir_inner(root, &p, files)?;
            }
        } else if p.is_file() {
            let rel = p
                .strip_prefix(root)
                .map_err(|e| ServiceError::Io(e.to_string()))?
                .to_string_lossy()
                .into_owned();
            if !rel.starts_with(".studyfrog/") {
                files.push(rel);
            }
        }
    }
    Ok(())
}

/// Find nearest stack folder enclosing `rel` path.
pub fn find_enclosing_stack(stacks: &[StackRow], rel: &str) -> Option<StackRow> {
    let mut dir = parent_folder(rel);
    loop {
        if let Some(s) = stacks.iter().find(|s| s.folder == dir) {
            return Some(s.clone());
        }
        if dir.is_empty() {
            return None;
        }
        dir = parent_folder(dir);
    }
}

/// Find all markdown files referencing any moved files and plan their link rewrites.
pub fn collect_link_rewrites(
    session: &VaultSession,
    mappings: &[(String, String)],
) -> Result<Vec<LinkRewrite>, ServiceError> {
    let all_files = walk_rel_files(&session.root, "")?;
    let mut rewrites = Vec::new();
    for rel in all_files {
        if !rel.to_lowercase().ends_with(".md") {
            continue;
        }
        let full = session.root.join(&rel);
        let text = fs::read_to_string(&full).map_err(|e| io_err(&rel, &e))?;
        let base_dir = parent_folder(&rel);
        let (new_text, changed) = rewrite_markdown_links(base_dir, &text, mappings);
        if changed {
            rewrites.push(LinkRewrite {
                path: rel,
                old_content: text,
                new_content: new_text,
            });
        }
    }
    Ok(rewrites)
}

/// Plan cross-stack move effects: items changing stack, `referenced_by` fix, and dropped memory.
#[allow(clippy::too_many_arguments)]
pub fn plan_cross_stack(
    session: &VaultSession,
    files: &[(String, String)],
    src_stack: &Option<StackRow>,
    dst_stack: &Option<StackRow>,
    items_count: &mut usize,
    sched_dropped: &mut usize,
    dropped_mem: &mut Vec<DroppedMemory>,
    doc_updates: &mut Vec<LinkRewrite>,
) -> Result<(), ServiceError> {
    let dst_stack_id = dst_stack.as_ref().map(|s| format_ulid(s.uid));
    let src_stack_uid = src_stack.as_ref().map_or(Uid(0), |s| s.uid);

    for (old_rel, new_rel) in files {
        if !old_rel.to_lowercase().ends_with(".md")
            || old_rel.rsplit('/').next() == Some(STACK_FILE)
        {
            continue;
        }
        let full = session.root.join(old_rel);
        let text = fs::read_to_string(&full).map_err(|e| io_err(old_rel, &e))?;
        let Ok(doc) = parse_doc(&text) else {
            continue;
        };
        let uid = match &doc {
            Doc::Note(d) => d.meta.uid,
            Doc::Card(d) => d.meta.uid,
            Doc::Question(d) => d.meta.uid,
            Doc::Stack(_) | Doc::Diary(_) => continue,
        };
        *items_count += 1;
        let refs = references_of(&doc);
        if let Some(target_id) = &dst_stack_id
            && refs.contains(target_id)
        {
            let filtered: Vec<String> = refs.into_iter().filter(|r| r != target_id).collect();
            let updated = with_references(&doc, &filtered);
            let rendered = render_doc(&updated)
                .map_err(|e| ServiceError::Invalid(format!("{old_rel}: {e}")))?;
            doc_updates.push(LinkRewrite {
                path: new_rel.clone(),
                old_content: text.clone(),
                new_content: rendered,
            });
        }
        let mem =
            load_mem_state_in(&session.conn, (uid, src_stack_uid)).map_err(ServiceError::Store)?;
        let sm2 =
            load_sm2_state_in(&session.conn, (uid, src_stack_uid)).map_err(ServiceError::Store)?;
        if mem.is_some() || sm2.is_some() {
            *sched_dropped += 1;
            dropped_mem.push((uid, src_stack_uid, mem, sm2));
        }
    }
    Ok(())
}
