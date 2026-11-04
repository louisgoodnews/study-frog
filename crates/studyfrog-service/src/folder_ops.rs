//! File and folder operations in the vault: create, rename, move, delete, rollback, and undo.

pub use crate::folder_ops_exec::{
    execute_create_folder, execute_delete, execute_delete_with_fault, execute_move,
    execute_move_with_fault, execute_rename, execute_rename_with_fault, undo_folder_op,
};
pub use crate::folder_ops_links::rewrite_markdown_links;
pub use crate::folder_ops_path::{
    check_not_root_or_essential, check_vault_rel, is_hostile_segment, is_subpath_or_equal,
    normalize_segments, sanitize_new_name, sanitize_rel_path,
};
pub use crate::folder_ops_plan::{plan_create_folder, plan_delete, plan_move, plan_rename};
pub use crate::folder_ops_types::{
    CreateFolderPlan, DeletePlan, FolderSnapshot, LinkRewrite, MovePlan, RenamePlan, UndoAction,
};
pub use crate::folder_ops_walk::walk_rel_files;

use crate::session::{ServiceError, VaultInfo, VaultSession};

/// Create a new folder inside the vault.
pub fn create_folder(
    session: &mut VaultSession,
    rel: &str,
) -> Result<(VaultInfo, FolderSnapshot, CreateFolderPlan), ServiceError> {
    let plan = plan_create_folder(session, rel)?;
    let (info, snap) = execute_create_folder(session, &plan)?;
    Ok((info, snap, plan))
}

/// Rename a file or folder within its parent directory, rewriting links in referencing files.
pub fn rename_path(
    session: &mut VaultSession,
    src: &str,
    new_name: &str,
) -> Result<(VaultInfo, FolderSnapshot, RenamePlan), ServiceError> {
    let plan = plan_rename(session, src, new_name)?;
    let (info, snap) = execute_rename(session, &plan)?;
    Ok((info, snap, plan))
}

/// Move a file or folder to a destination folder, updating primary stack, `referenced_by`,
/// link references, and dropping unreferenced SRS schedules.
pub fn move_path(
    session: &mut VaultSession,
    src: &str,
    dst_folder: &str,
) -> Result<(VaultInfo, FolderSnapshot, MovePlan), ServiceError> {
    let plan = plan_move(session, src, dst_folder)?;
    let (info, snap) = execute_move(session, &plan)?;
    Ok((info, snap, plan))
}

/// Delete a file or folder, returning an undo snapshot that can restore it.
pub fn delete_path(
    session: &mut VaultSession,
    target: &str,
) -> Result<(VaultInfo, FolderSnapshot, DeletePlan), ServiceError> {
    let plan = plan_delete(session, target)?;
    let (info, snap) = execute_delete(session, &plan)?;
    Ok((info, snap, plan))
}
