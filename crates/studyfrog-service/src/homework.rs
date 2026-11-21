//! Homework items: validated saves, status changes, and checklist ticks on top of the store.

use studyfrog_calendar::homework::{
    HomeworkItem, next_status, toggle_checklist_step, validate_homework,
};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::homework::{
    delete_homework, load_homework, load_homework_item, save_homework,
};
use studyfrog_store::homework_toml::homework_to_toml;

use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// All homework items, ordered by due date and time.
pub fn list_homework(session: &VaultSession) -> Result<Vec<HomeworkItem>, ServiceError> {
    load_homework(&session.conn).map_err(ServiceError::Store)
}

/// A fresh homework id.
pub fn new_homework_id(ids: IdSource) -> Uid {
    compose_ulid(ids.now_ms, ids.random)
}

/// The vault file that mirrors the homework items (`.studyfrog/homework.toml`).
pub const HOMEWORK_FILE: &str = ".studyfrog/homework.toml";

/// Write every homework item to the vault file, through a temporary file so a crash never leaves a
/// half-written file behind.
pub fn mirror_homework_file(session: &VaultSession) -> Result<(), ServiceError> {
    let items = load_homework(&session.conn).map_err(ServiceError::Store)?;
    let path = session.root.join(HOMEWORK_FILE);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| ServiceError::Io(format!("{}: {e}", dir.display())))?;
    }
    let temp = path.with_extension("toml.tmp");
    std::fs::write(&temp, homework_to_toml(&items))
        .map_err(|e| ServiceError::Io(format!("{}: {e}", temp.display())))?;
    std::fs::rename(&temp, &path).map_err(|e| ServiceError::Io(format!("{}: {e}", path.display())))
}

/// Validate and store a homework item; returns all homework items.
pub fn save_homework_checked(
    session: &mut VaultSession,
    item: HomeworkItem,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    validate_homework(&item).map_err(ServiceError::Invalid)?;
    save_homework(&session.conn, &item).map_err(ServiceError::Store)?;
    mirror_homework_file(session)?;
    list_homework(session)
}

/// Delete a homework item; returns the remaining ones.
pub fn delete_homework_checked(
    session: &mut VaultSession,
    id: Uid,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    if !delete_homework(&session.conn, id).map_err(ServiceError::Store)? {
        return Err(ServiceError::Invalid(
            "that homework item no longer exists".to_owned(),
        ));
    }
    mirror_homework_file(session)?;
    list_homework(session)
}

/// Advance an item's status one step (not started, in progress, completed, reopened).
pub fn advance_homework_status(
    session: &mut VaultSession,
    id: Uid,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    let item = load_item_or_err(session, id)?;
    let updated = HomeworkItem {
        status: next_status(item.status),
        ..item
    };
    save_homework(&session.conn, &updated).map_err(ServiceError::Store)?;
    mirror_homework_file(session)?;
    list_homework(session)
}

/// Tick or untick one checklist step of an item.
pub fn toggle_homework_step(
    session: &mut VaultSession,
    id: Uid,
    index: usize,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    let item = load_item_or_err(session, id)?;
    let updated = toggle_checklist_step(&item, index)
        .ok_or_else(|| ServiceError::Invalid("that checklist step does not exist".to_owned()))?;
    save_homework(&session.conn, &updated).map_err(ServiceError::Store)?;
    mirror_homework_file(session)?;
    list_homework(session)
}

fn load_item_or_err(session: &VaultSession, id: Uid) -> Result<HomeworkItem, ServiceError> {
    load_homework_item(&session.conn, id)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that homework item no longer exists".to_owned()))
}
