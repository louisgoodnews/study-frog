//! Calendar sessions: validated saves on top of the store.

use studyfrog_calendar::session::{Session, validate_session};
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::sessions::{delete_session, load_sessions, save_session};

use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// All sessions, ordered by start.
pub fn list_sessions(session: &VaultSession) -> Result<Vec<Session>, ServiceError> {
    load_sessions(&session.conn).map_err(ServiceError::Store)
}

/// A fresh session id.
pub fn new_session_id(ids: IdSource) -> Uid {
    compose_ulid(ids.now_ms, ids.random)
}

/// Validate and store a session (new or edited); returns all sessions.
pub fn save_session_checked(
    session: &mut VaultSession,
    s: Session,
) -> Result<Vec<Session>, ServiceError> {
    validate_session(&s).map_err(ServiceError::Invalid)?;
    save_session(&mut session.conn, &s).map_err(ServiceError::Store)?;
    list_sessions(session)
}

/// Delete a session; returns the remaining ones.
pub fn delete_session_checked(
    session: &mut VaultSession,
    id: Uid,
) -> Result<Vec<Session>, ServiceError> {
    if !delete_session(&session.conn, id).map_err(ServiceError::Store)? {
        return Err(ServiceError::Invalid(
            "that session no longer exists".to_owned(),
        ));
    }
    list_sessions(session)
}

/// Skip one occurrence of a repeating session without deleting the series.
pub fn cancel_occurrence(
    session: &mut VaultSession,
    id: Uid,
    date: Date,
) -> Result<Vec<Session>, ServiceError> {
    let mut all = list_sessions(session)?;
    let s = all
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| ServiceError::Invalid("that session no longer exists".to_owned()))?;
    if !s.cancelled.contains(&date) {
        s.cancelled.push(date);
        s.cancelled.sort();
    }
    let updated = s.clone();
    save_session(&mut session.conn, &updated).map_err(ServiceError::Store)?;
    list_sessions(session)
}
