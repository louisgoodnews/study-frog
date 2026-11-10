//! Timetable entries: validated saves on top of the store.

use studyfrog_calendar::timetable::{TimetableEntry, validate_timetable_entry};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::timetable::{
    delete_timetable_entry, load_timetable_entries, save_timetable_entry,
};

use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// All timetable entries, ordered by weekday and start time.
pub fn list_timetable_entries(session: &VaultSession) -> Result<Vec<TimetableEntry>, ServiceError> {
    load_timetable_entries(&session.conn).map_err(ServiceError::Store)
}

/// A fresh timetable entry id.
pub fn new_timetable_id(ids: IdSource) -> Uid {
    compose_ulid(ids.now_ms, ids.random)
}

/// Validate and store a timetable entry; returns all timetable entries.
pub fn save_timetable_entry_checked(
    session: &mut VaultSession,
    entry: TimetableEntry,
) -> Result<Vec<TimetableEntry>, ServiceError> {
    validate_timetable_entry(&entry).map_err(ServiceError::Invalid)?;
    save_timetable_entry(&session.conn, &entry).map_err(ServiceError::Store)?;
    list_timetable_entries(session)
}

/// Delete a timetable entry by id; returns the remaining ones.
pub fn delete_timetable_entry_checked(
    session: &mut VaultSession,
    id: Uid,
) -> Result<Vec<TimetableEntry>, ServiceError> {
    if !delete_timetable_entry(&session.conn, id).map_err(ServiceError::Store)? {
        return Err(ServiceError::Invalid(
            "that timetable entry no longer exists".to_owned(),
        ));
    }
    list_timetable_entries(session)
}
