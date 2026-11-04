//! Atomic execution, rollback, and undo for folder and file operations.

use std::fs;

use rusqlite::params;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::poll::prime;
use studyfrog_store::reindex::reindex;

use crate::create::write_new;
use crate::folder_ops_rollback::rollback;
use crate::folder_ops_types::{
    CreateFolderPlan, DeletePlan, FolderSnapshot, MovePlan, RenamePlan, UndoAction,
};
use crate::session::{ServiceError, VaultInfo, VaultSession, vault_info};

fn io_err(path: &str, err: &std::io::Error) -> ServiceError {
    ServiceError::Io(format!("{path}: {err}"))
}

fn refresh(session: &mut VaultSession) -> Result<VaultInfo, ServiceError> {
    reindex(&mut session.conn, &session.root).map_err(ServiceError::Store)?;
    session.poll = prime(&session.root).map_err(ServiceError::Store)?;
    vault_info(&session.root, &session.conn)
}

/// Execute folder creation.
pub fn execute_create_folder(
    session: &mut VaultSession,
    plan: &CreateFolderPlan,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    let full = session.root.join(&plan.path);
    fs::create_dir_all(&full).map_err(|e| io_err(&plan.path, &e))?;
    let snap = FolderSnapshot(vec![UndoAction::DeleteDir(plan.path.clone())]);
    let info = refresh(session)?;
    Ok((info, snap))
}

/// Execute renaming a folder or file.
pub fn execute_rename(
    session: &mut VaultSession,
    plan: &RenamePlan,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    execute_rename_with_fault(session, plan, None)
}

/// Execute rename with optional fault injection for rollback testing.
pub fn execute_rename_with_fault(
    session: &mut VaultSession,
    plan: &RenamePlan,
    fault_step: Option<usize>,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    let mut actions = Vec::new();
    let mut step = 0;

    let res: Result<(), ServiceError> = (|| {
        let (src_full, dst_full) = (session.root.join(&plan.src), session.root.join(&plan.dst));
        if let Some(parent) = dst_full.parent() {
            fs::create_dir_all(parent).map_err(|e| io_err(&plan.dst, &e))?;
        }
        step += 1;
        if fault_step == Some(step) {
            return Err(ServiceError::Invalid(
                "simulated fault before rename".into(),
            ));
        }
        fs::rename(&src_full, &dst_full).map_err(|e| io_err(&plan.src, &e))?;
        actions.push(UndoAction::MovePath {
            from: plan.dst.clone(),
            to: plan.src.clone(),
        });
        for rw in &plan.link_rewrites {
            step += 1;
            if fault_step == Some(step) {
                return Err(ServiceError::Invalid(
                    "simulated fault during link rewrite".into(),
                ));
            }
            actions.push(UndoAction::WriteFile(
                rw.path.clone(),
                rw.old_content.clone(),
            ));
            write_new(&session.root.join(&rw.path), &rw.new_content)?;
        }
        Ok(())
    })();

    if let Err(e) = res {
        rollback(&session.root, &session.conn, &actions);
        return Err(e);
    }
    let info = refresh(session)?;
    Ok((info, FolderSnapshot(actions)))
}

/// Execute moving a folder or file.
pub fn execute_move(
    session: &mut VaultSession,
    plan: &MovePlan,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    execute_move_with_fault(session, plan, None)
}

/// Execute move with optional fault injection for rollback testing.
pub fn execute_move_with_fault(
    session: &mut VaultSession,
    plan: &MovePlan,
    fault_step: Option<usize>,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    let mut actions = Vec::new();
    let mut step = 0;

    let res: Result<(), ServiceError> = (|| {
        let (src_full, dst_full) = (session.root.join(&plan.src), session.root.join(&plan.dst));
        if let Some(parent) = dst_full.parent() {
            fs::create_dir_all(parent).map_err(|e| io_err(&plan.dst, &e))?;
        }
        step += 1;
        if fault_step == Some(step) {
            return Err(ServiceError::Invalid("simulated fault before move".into()));
        }
        fs::rename(&src_full, &dst_full).map_err(|e| io_err(&plan.src, &e))?;
        actions.push(UndoAction::MovePath {
            from: plan.dst.clone(),
            to: plan.src.clone(),
        });
        for update in &plan.doc_updates {
            step += 1;
            if fault_step == Some(step) {
                return Err(ServiceError::Invalid(
                    "simulated fault during doc update".into(),
                ));
            }
            actions.push(UndoAction::WriteFile(
                update.path.clone(),
                update.old_content.clone(),
            ));
            write_new(&session.root.join(&update.path), &update.new_content)?;
        }
        for rw in &plan.link_rewrites {
            step += 1;
            if fault_step == Some(step) {
                return Err(ServiceError::Invalid(
                    "simulated fault during link rewrite".into(),
                ));
            }
            actions.push(UndoAction::WriteFile(
                rw.path.clone(),
                rw.old_content.clone(),
            ));
            write_new(&session.root.join(&rw.path), &rw.new_content)?;
        }
        for (item_uid, stack_uid, mem, sm2) in &plan.dropped_memory {
            step += 1;
            if fault_step == Some(step) {
                return Err(ServiceError::Invalid(
                    "simulated fault during memory drop".into(),
                ));
            }
            actions.push(UndoAction::RestoreMemory {
                item_uid: *item_uid,
                stack_uid: *stack_uid,
                mem: *mem,
                sm2: *sm2,
            });
            let item_str = format_ulid(*item_uid);
            let stack_str = format_ulid(*stack_uid);
            let _ = session.conn.execute(
                "DELETE FROM mem_state WHERE item_uid = ?1 AND stack_uid = ?2",
                params![item_str, stack_str],
            );
            let _ = session.conn.execute(
                "DELETE FROM sm2_state WHERE item_uid = ?1 AND stack_uid = ?2",
                params![item_str, stack_str],
            );
        }
        Ok(())
    })();

    if let Err(e) = res {
        rollback(&session.root, &session.conn, &actions);
        return Err(e);
    }
    let info = refresh(session)?;
    Ok((info, FolderSnapshot(actions)))
}

/// Execute deleting a folder or file.
pub fn execute_delete(
    session: &mut VaultSession,
    plan: &DeletePlan,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    execute_delete_with_fault(session, plan, None)
}

/// Execute delete with optional fault injection for rollback testing.
pub fn execute_delete_with_fault(
    session: &mut VaultSession,
    plan: &DeletePlan,
    fault_step: Option<usize>,
) -> Result<(VaultInfo, FolderSnapshot), ServiceError> {
    let mut actions = Vec::new();
    let mut step = 0;

    let res: Result<(), ServiceError> = (|| {
        for rel in &plan.files_to_delete {
            let full = session.root.join(rel);
            let bytes = fs::read(&full).map_err(|e| io_err(rel, &e))?;
            actions.push(UndoAction::RecreateFile(rel.clone(), bytes));
        }
        step += 1;
        if fault_step == Some(step) {
            return Err(ServiceError::Invalid(
                "simulated fault before delete".into(),
            ));
        }
        let target_full = session.root.join(&plan.target);
        if plan.is_folder {
            fs::remove_dir_all(&target_full).map_err(|e| io_err(&plan.target, &e))?;
        } else {
            fs::remove_file(&target_full).map_err(|e| io_err(&plan.target, &e))?;
        }
        Ok(())
    })();

    if let Err(e) = res {
        rollback(&session.root, &session.conn, &actions);
        return Err(e);
    }
    let info = refresh(session)?;
    Ok((info, FolderSnapshot(actions)))
}

/// Revert all operations recorded in a `FolderSnapshot`.
pub fn undo_folder_op(
    session: &mut VaultSession,
    snapshot: &FolderSnapshot,
) -> Result<VaultInfo, ServiceError> {
    rollback(&session.root, &session.conn, &snapshot.0);
    refresh(session)
}
