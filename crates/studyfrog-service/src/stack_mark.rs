//! Marking a folder as a stack, and turning a stack into a normal folder (ADR 0011, TASK-0217).

use std::fs;
use std::path::Path;

use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;
use studyfrog_md::doc::{Doc, StackDoc};
use studyfrog_md::refs::{references_of, with_references};
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::{compose_ulid, format_ulid};
use studyfrog_store::stackmap::STACK_FILE;

use crate::bulk::{Snapshot, bulk_undo};
use crate::create::{IdSource, check_title, refresh, write_new};
use crate::create_more::Created;
use crate::session::{ServiceError, VaultInfo, VaultSession};

pub use crate::stack_mark_plan::{UnmarkPlan, plan_unmark};

fn io(path: &str, err: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {err}"))
}

/// Turn a stack into a normal folder: delete `_stack.md` and `_stack.toml`, remove references, and reindex.
pub fn unmark_stack(
    session: &mut VaultSession,
    folder: &str,
) -> Result<(VaultInfo, Snapshot, UnmarkPlan), ServiceError> {
    let plan = plan_unmark(session, folder)?;
    let mut snap = Snapshot::default();

    let res: Result<(), ServiceError> = (|| {
        let stack_id_str = format_ulid(plan.uid);
        for (_, rel) in &plan.referencing_items {
            let abs = session.root.join(rel);
            let text = fs::read_to_string(&abs).map_err(|e| io(rel, &e))?;
            snap.0.push((rel.clone(), Some(text.clone())));
            let doc = studyfrog_md::parse::parse_doc(&text)
                .map_err(|e| ServiceError::Invalid(format!("could not parse {rel}: {e}")))?;
            let refs: Vec<String> = references_of(&doc)
                .into_iter()
                .filter(|id| id != &stack_id_str)
                .collect();
            let updated = with_references(&doc, &refs);
            let rendered =
                render_doc(&updated).map_err(|e| ServiceError::Invalid(format!("{rel}: {e}")))?;
            write_new(&abs, &rendered)?;
        }

        if plan.has_settings {
            let toml_rel = format!("{folder}/_stack.toml");
            let abs = session.root.join(&toml_rel);
            let text = fs::read_to_string(&abs).map_err(|e| io(&toml_rel, &e))?;
            snap.0.push((toml_rel.clone(), Some(text)));
            fs::remove_file(&abs).map_err(|e| io(&toml_rel, &e))?;
        }

        let stack_rel = format!("{folder}/{STACK_FILE}");
        let abs = session.root.join(&stack_rel);
        let text = fs::read_to_string(&abs).map_err(|e| io(&stack_rel, &e))?;
        snap.0.push((stack_rel.clone(), Some(text)));
        fs::remove_file(&abs).map_err(|e| io(&stack_rel, &e))?;

        Ok(())
    })();

    if let Err(e) = res {
        let _ = bulk_undo(session, &snap);
        return Err(e);
    }

    let info = refresh(session)?;
    Ok((info, snap, plan))
}

/// Folder name from path (the last component).
fn folder_name(folder: &str) -> &str {
    folder.rsplit_once('/').map_or(folder, |(_, name)| name)
}

/// Turn an existing normal folder into a stack by writing `_stack.md`.
pub fn mark_as_stack(
    session: &mut VaultSession,
    folder: &str,
    ids: IdSource,
) -> Result<(VaultInfo, Snapshot, Created), ServiceError> {
    let dir = session.root.join(folder);
    if !dir.is_dir() {
        return Err(ServiceError::Invalid(format!(
            "\"{folder}\" is not a directory"
        )));
    }
    let stack_rel = format!("{folder}/{STACK_FILE}");
    let stack_file = dir.join(STACK_FILE);
    if stack_file.is_file() {
        return Err(ServiceError::Invalid(format!(
            "\"{folder}\" is already a stack"
        )));
    }

    let title = folder_name(folder);
    check_title(title)?;

    let uid = compose_ulid(ids.now_ms, ids.random);
    let doc = Doc::Stack(StackDoc {
        meta: Meta {
            uid,
            title: title.to_owned(),
            tags: Vec::new(),
            aliases: Vec::new(),
            created: Some(UnixMs(i64::try_from(ids.now_ms).unwrap_or(i64::MAX))),
            updated: None,
        },
        description: String::new(),
        body: String::new(),
        extra: Vec::new(),
    });

    let text = render_doc(&doc).map_err(|e| ServiceError::Invalid(format!("{stack_rel}: {e}")))?;
    let snap = Snapshot(vec![(stack_rel.clone(), None)]);
    write_new(&stack_file, &text)?;

    let info = refresh(session)?;
    Ok((
        info,
        snap,
        Created {
            title: title.to_owned(),
            path: stack_rel,
            uid: Some(uid),
        },
    ))
}

/// Revert an unmark or mark operation.
pub fn undo_mark_change(
    session: &mut VaultSession,
    snapshot: &Snapshot,
) -> Result<VaultInfo, ServiceError> {
    bulk_undo(session, snapshot)
}

/// List relative directory paths in the vault that are normal folders (not marked with `_stack.md`).
pub fn list_normal_folders(session: &VaultSession) -> Result<Vec<String>, ServiceError> {
    let mut out = Vec::new();
    walk_normal(&session.root, &session.root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_normal(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), ServiceError> {
    let entries = fs::read_dir(dir).map_err(|e| ServiceError::Io(e.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|e| ServiceError::Io(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "wiki" {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .map_err(|e| ServiceError::Io(e.to_string()))?
                .to_string_lossy()
                .into_owned();
            if !path.join(STACK_FILE).is_file() {
                out.push(rel);
            }
            walk_normal(root, &path, out)?;
        }
    }
    Ok(())
}
