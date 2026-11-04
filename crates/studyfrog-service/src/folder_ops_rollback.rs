//! Rollback and undo actions for folder and file operations.

use std::fs;
use std::path::Path;

use studyfrog_store::schedule::{save_mem_state_in, save_sm2_state_in};

use crate::create::write_new;
use crate::folder_ops_types::UndoAction;

/// Apply one undo action against vault filesystem and database.
pub fn apply_undo_action(root: &Path, conn: &rusqlite::Connection, action: &UndoAction) {
    match action {
        UndoAction::DeleteDir(rel) => {
            let _ = fs::remove_dir_all(root.join(rel));
        }
        UndoAction::DeleteFile(rel) => {
            let _ = fs::remove_file(root.join(rel));
        }
        UndoAction::MovePath { from, to } => {
            let (f, t) = (root.join(from), root.join(to));
            if let Some(parent) = t.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::rename(f, t);
        }
        UndoAction::WriteFile(rel, content) => {
            let _ = write_new(&root.join(rel), content);
        }
        UndoAction::RestoreMemory {
            item_uid,
            stack_uid,
            mem,
            sm2,
        } => {
            if let Some(m) = mem {
                let _ = save_mem_state_in(conn, (*item_uid, *stack_uid), m);
            }
            if let Some(s) = sm2 {
                let _ = save_sm2_state_in(conn, (*item_uid, *stack_uid), s);
            }
        }
        UndoAction::RecreateFile(rel, bytes) => {
            let p = root.join(rel);
            if let Some(parent) = p.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(p, bytes);
        }
    }
}

/// Roll back all performed actions in reverse order.
pub fn rollback(root: &Path, conn: &rusqlite::Connection, actions: &[UndoAction]) {
    for action in actions.iter().rev() {
        apply_undo_action(root, conn, action);
    }
}
