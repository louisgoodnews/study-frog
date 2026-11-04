//! Executing folder effects against the vault filesystem.

use std::fs;
use std::path::Path;

use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{Doc, NoteDoc};
use studyfrog_md::render::render_doc;
use studyfrog_service::create::{fresh_id_source, slug};
use studyfrog_service::folder_ops::{execute_delete, execute_move, plan_delete, plan_move};
use studyfrog_service::session::{VaultInfo, VaultSession, vault_info};
use studyfrog_service::stack_mark::mark_as_stack;

use crate::exec::Worker;
use crate::folders_types::{EntryKind, FolderEffect, FolderEntry, FoldersMsg};
use crate::msg::Msg;

/// Execute a folder effect.
pub fn execute(w: &mut Worker, effect: FolderEffect, now_ns: i64) -> Vec<Msg> {
    let Some(session) = &mut w.session else {
        return vec![Msg::Folders(FoldersMsg::OpDone(
            Err("no vault open".into()),
        ))];
    };

    match effect {
        FolderEffect::Scan => {
            let entries = scan_vault(&session.root);
            vec![Msg::Folders(FoldersMsg::Scanned(entries))]
        }
        FolderEffect::CreateNote { folder, title } => {
            let res = create_note(session, &folder, &title, now_ns);
            vec![Msg::Folders(FoldersMsg::OpDone(res))]
        }
        FolderEffect::CreateFolder {
            parent,
            name,
            is_stack,
        } => {
            let rel = if parent.is_empty() {
                slug(&name)
            } else {
                format!("{parent}/{}", slug(&name))
            };
            let dir = session.root.join(&rel);
            let res = fs::create_dir_all(&dir)
                .map_err(|e| e.to_string())
                .and_then(|()| {
                    if is_stack {
                        let now_ms = (now_ns / 1_000_000) as u64;
                        let ids = fresh_id_source(now_ms).map_err(|e| e.to_string())?;
                        mark_as_stack(session, &rel, ids)
                            .map(|(info, _, _)| info)
                            .map_err(|e| e.to_string())
                    } else {
                        vault_info(&session.root, &session.conn).map_err(|e| e.to_string())
                    }
                });
            vec![Msg::Folders(FoldersMsg::OpDone(res))]
        }
        FolderEffect::Rename { src, new_name } => {
            let res = studyfrog_service::folder_ops::rename_path(session, &src, &new_name)
                .map(|(info, _, _)| info)
                .map_err(|e| e.to_string());
            vec![Msg::Folders(FoldersMsg::OpDone(res))]
        }
        FolderEffect::PlanMove { src, dst_folder } => {
            let res = plan_move(session, &src, &dst_folder).map_err(|e| e.to_string());
            vec![Msg::Folders(FoldersMsg::MovePlanReady(res))]
        }
        FolderEffect::ExecuteMove(plan) => {
            let res = execute_move(session, &plan)
                .map(|(info, _)| info)
                .map_err(|e| e.to_string());
            vec![Msg::Folders(FoldersMsg::OpDone(res))]
        }
        FolderEffect::PlanDelete(target) => {
            let res = plan_delete(session, &target).map_err(|e| e.to_string());
            vec![Msg::Folders(FoldersMsg::DeletePlanReady(res))]
        }
        FolderEffect::ExecuteDelete(plan) => {
            let res = execute_delete(session, &plan)
                .map(|(info, _)| info)
                .map_err(|e| e.to_string());
            vec![Msg::Folders(FoldersMsg::OpDone(res))]
        }
        FolderEffect::MarkStack(folder) => {
            let now_ms = (now_ns / 1_000_000) as u64;
            let res = fresh_id_source(now_ms)
                .map_err(|e| e.to_string())
                .and_then(|ids| mark_as_stack(session, &folder, ids).map_err(|e| e.to_string()))
                .map(|(info, _, _)| info);
            vec![Msg::Folders(FoldersMsg::OpDone(res))]
        }
    }
}

fn create_note(
    session: &mut VaultSession,
    folder: &str,
    title: &str,
    now_ns: i64,
) -> Result<VaultInfo, String> {
    let now_ms = now_ns / 1_000_000;
    let file_slug = slug(title);
    let filename = format!("{file_slug}.md");
    let rel_path = if folder.is_empty() {
        filename
    } else {
        format!("{folder}/{filename}")
    };
    let target = session.root.join(&rel_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let doc = Doc::Note(NoteDoc {
        meta: Meta {
            title: title.to_owned(),
            uid: Uid(0),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: Some(UnixMs(now_ms)),
            updated: Some(UnixMs(now_ms)),
        },
        rehearse_title: false,
        body: String::new(),
        extra: Vec::new(),
    });
    let rendered = render_doc(&doc).map_err(|e| e.to_string())?;
    fs::write(&target, rendered).map_err(|e| e.to_string())?;
    vault_info(&session.root, &session.conn).map_err(|e| e.to_string())
}

fn scan_vault(root: &Path) -> Result<Vec<FolderEntry>, String> {
    let mut entries = Vec::new();
    scan_dir(root, root, "", 0, &mut entries)?;
    Ok(entries)
}

fn scan_dir(
    _root: &Path,
    dir: &Path,
    rel_parent: &str,
    depth: usize,
    out: &mut Vec<FolderEntry>,
) -> Result<(), String> {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for entry in read.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == ".studyfrog" {
            continue;
        }
        let rel = if rel_parent.is_empty() {
            name.clone()
        } else {
            format!("{rel_parent}/{name}")
        };
        let mtime_ns = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_nanos() as i64);

        if path.is_dir() {
            let is_stack = path.join("_stack.md").is_file();
            out.push(FolderEntry {
                rel_path: rel.clone(),
                name: name.clone(),
                is_folder: true,
                is_stack,
                parent: rel_parent.to_owned(),
                depth,
                kind: EntryKind::Folder,
                mtime_ns,
            });
            scan_dir(_root, &path, &rel, depth + 1, out)?;
        } else if path.is_file() {
            if name == "_stack.md" {
                continue;
            }
            let kind = if name.ends_with(".md") {
                EntryKind::Markdown
            } else if name.ends_with(".pdf") {
                EntryKind::Pdf
            } else if name.ends_with(".png")
                || name.ends_with(".jpg")
                || name.ends_with(".jpeg")
                || name.ends_with(".gif")
                || name.ends_with(".webp")
            {
                EntryKind::Image
            } else {
                let ext = name
                    .rsplit_once('.')
                    .map_or("DOC", |(_, e)| e)
                    .to_ascii_uppercase();
                EntryKind::Other(ext)
            };
            out.push(FolderEntry {
                rel_path: rel,
                name,
                is_folder: false,
                is_stack: false,
                parent: rel_parent.to_owned(),
                depth,
                kind,
                mtime_ns,
            });
        }
    }
    Ok(())
}
