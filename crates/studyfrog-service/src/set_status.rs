//! Changing the status an item or a stack states, without opening the editor.

use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::status::with_status;

use crate::document::{load_document, save_document};
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// Set the status of `uid` (`None` leaves it to the history) and return the refreshed vault.
pub fn set_status(
    session: &mut VaultSession,
    uid: Uid,
    status: Option<Status>,
    now: UnixMs,
) -> Result<VaultInfo, ServiceError> {
    let loaded = load_document(session, uid)?;
    let doc = with_status(&loaded.doc, status);
    save_document(session, &loaded.path, &doc, loaded.hash, now).map(|s| s.info)
}
