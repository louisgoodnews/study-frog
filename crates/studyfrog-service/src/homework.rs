//! Homework assignments: validated CRUD and atomic `.studyfrog/homework.toml` + SQLite sync.

use std::path::Path;

use rusqlite::Connection;
use studyfrog_calendar::homework::{
    ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus, normalize_homework,
    toggle_checklist_item, toggle_homework_completed, validate_homework,
};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::homework::get_homework;
use studyfrog_store::homework_toml::{
    delete_homework_synced, save_homework_synced, sync_homework_from_toml,
};

use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// Inputs needed to create a new homework item before an identity is assigned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeworkDraft {
    /// Single-line assignment title.
    pub title: String,
    /// Optional subject or course name.
    pub subject: Option<String>,
    /// Optional linked stack id.
    pub stack_uid: Option<Uid>,
    /// Optional linked timetable entry id.
    pub timetable_id: Option<Uid>,
    /// Local due date and minute of day.
    pub due: LocalTime,
    /// Optional planned start date.
    pub start_date: Option<Date>,
    /// Positive estimated effort in minutes.
    pub estimated_minutes: u32,
    /// Priority level.
    pub priority: HomeworkPriority,
    /// Initial lifecycle status.
    pub status: HomeworkStatus,
    /// Ordered checklist sub-tasks.
    pub checklist: Vec<ChecklistItem>,
    /// Free-form notes.
    pub notes: String,
    /// Reminder lead times in minutes before `due`.
    pub reminders: Vec<u32>,
}

/// Construct a [`HomeworkItem`] from a generated [`Uid`] and a [`HomeworkDraft`].
pub fn homework_from_draft(id: Uid, draft: HomeworkDraft) -> HomeworkItem {
    HomeworkItem {
        id,
        title: draft.title,
        subject: draft.subject,
        stack_uid: draft.stack_uid,
        timetable_id: draft.timetable_id,
        due: draft.due,
        start_date: draft.start_date,
        estimated_minutes: draft.estimated_minutes,
        priority: draft.priority,
        status: draft.status,
        checklist: draft.checklist,
        notes: draft.notes,
        reminders: draft.reminders,
    }
}

/// Generate a fresh ULID for a homework item from an [`IdSource`].
pub fn new_homework_id(ids: IdSource) -> Uid {
    compose_ulid(ids.now_ms, ids.random)
}

/// List every homework item in canonical due-date order.
pub fn list_homework(session: &VaultSession) -> Result<Vec<HomeworkItem>, ServiceError> {
    studyfrog_store::homework::list_homework(&session.conn).map_err(ServiceError::Store)
}

/// Create, validate, and persist a new homework item; returns the updated list.
pub fn create_homework(
    session: &mut VaultSession,
    draft: HomeworkDraft,
    ids: IdSource,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    let item = homework_from_draft(new_homework_id(ids), draft);
    save_homework(session, item)
}

/// Validate, normalize, and atomically persist a homework item to SQLite and `.studyfrog/homework.toml`.
pub fn save_homework(
    session: &mut VaultSession,
    item: HomeworkItem,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    validate_homework(&item).map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let normalized = normalize_homework(item);
    save_homework_synced(&mut session.conn, &session.root, &normalized).map_err(ServiceError::Store)
}

/// Delete a homework item from SQLite and `.studyfrog/homework.toml`; returns remaining items.
pub fn delete_homework(
    session: &mut VaultSession,
    id: Uid,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    delete_homework_synced(&mut session.conn, &session.root, id)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that homework item no longer exists".to_owned()))
}

/// Toggle completion status of a homework item and persist both SQLite and `.studyfrog/homework.toml`.
pub fn toggle_homework(
    session: &mut VaultSession,
    id: Uid,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    let item = get_homework(&session.conn, id)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that homework item no longer exists".to_owned()))?;
    let updated = toggle_homework_completed(item);
    save_homework(session, updated)
}

/// Toggle one checklist entry of a homework item and persist both SQLite and `.studyfrog/homework.toml`.
pub fn toggle_homework_checklist(
    session: &mut VaultSession,
    id: Uid,
    index: usize,
) -> Result<Vec<HomeworkItem>, ServiceError> {
    let item = get_homework(&session.conn, id)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("that homework item no longer exists".to_owned()))?;
    if index >= item.checklist.len() {
        return Err(ServiceError::Invalid(
            "that checklist item does not exist".to_owned(),
        ));
    }
    let updated = toggle_checklist_item(item, index);
    save_homework(session, updated)
}

/// Reconcile `.studyfrog/homework.toml` and SQLite and return the resulting homework list.
pub fn sync_homework(session: &mut VaultSession) -> Result<Vec<HomeworkItem>, ServiceError> {
    sync_homework_on_open(&mut session.conn, &session.root)?;
    list_homework(session)
}

/// Sync `.studyfrog/homework.toml` into SQLite on vault open (backfilling TOML on restore).
pub(crate) fn sync_homework_on_open(
    conn: &mut Connection,
    root: &Path,
) -> Result<(), ServiceError> {
    sync_homework_from_toml(conn, root)
        .map(|_| ())
        .map_err(ServiceError::Store)
}
