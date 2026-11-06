//! Pure planning for creating, renaming, moving, and deleting folders and files.

use std::fs;

use studyfrog_md::doc::Doc;
use studyfrog_md::parse::parse_doc;
use studyfrog_store::overview::load_overview;
use studyfrog_store::stackmap::{STACK_FILE, parent_folder};

use crate::folder_ops_path::{
    check_not_root_or_essential, is_subpath_or_equal, sanitize_new_name, sanitize_rel_path,
};
use crate::folder_ops_types::{CreateFolderPlan, DeletePlan, MovePlan, RenamePlan};
use crate::folder_ops_walk::{
    collect_link_rewrites, find_enclosing_stack, plan_cross_stack, walk_rel_files,
};
use crate::session::{ServiceError, VaultSession};

fn io_err(path: &str, err: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {err}"))
}

/// Plan creation of a new folder.
pub fn plan_create_folder(
    session: &VaultSession,
    rel: &str,
) -> Result<CreateFolderPlan, ServiceError> {
    let clean = sanitize_rel_path(rel)?;
    check_not_root_or_essential(&clean)?;
    let target = session.root.join(&clean);
    if target.exists() {
        return Err(ServiceError::Invalid(format!("\"{clean}\" already exists")));
    }
    Ok(CreateFolderPlan { path: clean })
}

/// Plan renaming a folder or file within its parent directory.
pub fn plan_rename(
    session: &VaultSession,
    src: &str,
    new_name: &str,
) -> Result<RenamePlan, ServiceError> {
    let clean_src = sanitize_rel_path(src)?;
    check_not_root_or_essential(&clean_src)?;
    let clean_name = sanitize_new_name(new_name)?;
    let src_path = session.root.join(&clean_src);
    if !src_path.exists() {
        return Err(ServiceError::Invalid(format!(
            "\"{clean_src}\" does not exist"
        )));
    }
    let parent = parent_folder(&clean_src);
    let dst = if parent.is_empty() {
        clean_name
    } else {
        format!("{parent}/{clean_name}")
    };
    if dst == clean_src {
        return Err(ServiceError::Invalid(
            "source and destination names are the same".into(),
        ));
    }
    if session.root.join(&dst).exists() {
        return Err(ServiceError::Invalid(format!("\"{dst}\" already exists")));
    }
    let is_folder = src_path.is_dir();
    let mut files = Vec::new();
    if is_folder {
        let inside = walk_rel_files(&session.root, &clean_src)?;
        for file in inside {
            let sub = file[clean_src.len() + 1..].to_owned();
            files.push((file, format!("{dst}/{sub}")));
        }
    } else {
        files.push((clean_src.clone(), dst.clone()));
    }
    let link_rewrites = collect_link_rewrites(session, &files)?;
    Ok(RenamePlan {
        src: clean_src,
        dst,
        is_folder,
        files,
        link_rewrites,
    })
}

/// Plan moving a folder or file to a destination folder.
pub fn plan_move(
    session: &VaultSession,
    src: &str,
    dst_folder: &str,
) -> Result<MovePlan, ServiceError> {
    let clean_src = sanitize_rel_path(src)?;
    check_not_root_or_essential(&clean_src)?;
    let clean_dst_folder = if dst_folder.trim().is_empty() {
        String::new()
    } else {
        sanitize_rel_path(dst_folder)?
    };
    let src_path = session.root.join(&clean_src);
    if !src_path.exists() {
        return Err(ServiceError::Invalid(format!(
            "\"{clean_src}\" does not exist"
        )));
    }
    let is_folder = src_path.is_dir();
    if !clean_dst_folder.is_empty() {
        let dst_dir_path = session.root.join(&clean_dst_folder);
        if !dst_dir_path.is_dir() {
            return Err(ServiceError::Invalid(format!(
                "\"{clean_dst_folder}\" is not a directory"
            )));
        }
    }
    if is_folder && is_subpath_or_equal(&clean_src, &clean_dst_folder) {
        return Err(ServiceError::Invalid(
            "cannot move a folder into itself or its subfolder".into(),
        ));
    }
    let name = clean_src
        .rsplit_once('/')
        .map_or(clean_src.as_str(), |(_, n)| n);
    let dst = if clean_dst_folder.is_empty() {
        name.to_owned()
    } else {
        format!("{clean_dst_folder}/{name}")
    };
    if dst == clean_src {
        return Err(ServiceError::Invalid(
            "source and destination are identical".into(),
        ));
    }
    if session.root.join(&dst).exists() {
        return Err(ServiceError::Invalid(format!("\"{dst}\" already exists")));
    }
    let mut files = Vec::new();
    if is_folder {
        for file in walk_rel_files(&session.root, &clean_src)? {
            let sub = file[clean_src.len() + 1..].to_owned();
            files.push((file, format!("{dst}/{sub}")));
        }
    } else {
        files.push((clean_src.clone(), dst.clone()));
    }
    let stacks = load_overview(&session.conn)
        .map_err(ServiceError::Store)?
        .stacks;
    let src_stack = find_enclosing_stack(&stacks, &clean_src);
    let dst_stack = find_enclosing_stack(&stacks, &dst);
    let src_is_stack = stacks.iter().any(|s| s.folder == clean_src);
    let is_cross_stack = !src_is_stack
        && src_stack.as_ref().map(|s| &s.folder) != dst_stack.as_ref().map(|s| &s.folder);
    let (mut items_count, mut sched_dropped) = (0, 0);
    let mut dropped_mem = Vec::new();
    let mut doc_updates = Vec::new();

    if is_cross_stack {
        plan_cross_stack(
            session,
            &files,
            &src_stack,
            &dst_stack,
            &mut items_count,
            &mut sched_dropped,
            &mut dropped_mem,
            &mut doc_updates,
        )?;
    }
    let link_rewrites = collect_link_rewrites(session, &files)?;
    Ok(MovePlan {
        src: clean_src,
        dst,
        is_folder,
        files,
        is_cross_stack,
        source_stack: src_stack.as_ref().map(|s| s.folder.clone()),
        destination_stack: dst_stack.as_ref().map(|s| s.folder.clone()),
        source_stack_title: src_stack
            .as_ref()
            .map_or("Inbox".to_owned(), |s| s.title.clone()),
        destination_stack_title: dst_stack
            .as_ref()
            .map_or("Inbox".to_owned(), |s| s.title.clone()),
        items_changing_primary_stack: items_count,
        schedules_dropped: sched_dropped,
        dropped_memory: dropped_mem,
        doc_updates,
        link_rewrites,
    })
}

/// Plan deleting a folder or file.
pub fn plan_delete(session: &VaultSession, target: &str) -> Result<DeletePlan, ServiceError> {
    let clean = sanitize_rel_path(target)?;
    check_not_root_or_essential(&clean)?;
    let full = session.root.join(&clean);
    if !full.exists() {
        return Err(ServiceError::Invalid(format!("\"{clean}\" does not exist")));
    }
    let is_folder = full.is_dir();
    let files_to_delete = if is_folder {
        walk_rel_files(&session.root, &clean)?
    } else {
        vec![clean.clone()]
    };
    let mut items_count = 0;
    let mut stacks_count = 0;
    for rel in &files_to_delete {
        if rel.rsplit('/').next() == Some(STACK_FILE) {
            stacks_count += 1;
        } else if rel.to_lowercase().ends_with(".md") {
            let text = fs::read_to_string(session.root.join(rel)).map_err(|e| io_err(rel, &e))?;
            if let Ok(doc) = parse_doc(&text)
                && !matches!(doc, Doc::Stack(_))
            {
                items_count += 1;
            }
        }
    }
    Ok(DeletePlan {
        target: clean,
        is_folder,
        files_count: files_to_delete.len(),
        items_count,
        stacks_count,
        files_to_delete,
    })
}
