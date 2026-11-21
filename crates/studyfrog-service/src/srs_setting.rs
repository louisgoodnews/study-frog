//! Changing a scheduling switch of the vault from the app: the `[srs]` line in `studyfrog.toml` is
//! written, the rest of the file kept, and the vault read again (STORY-0177).

use studyfrog_config::retention_edit::with_setting_line;

use crate::session::{ServiceError, VaultInfo, VaultSession, vault_info};

/// Write `calibrated_start = on` under `[srs]`, reindex, and return the refreshed vault.
pub fn apply_calibrated_start(
    session: &mut VaultSession,
    on: bool,
) -> Result<VaultInfo, ServiceError> {
    let path = session.root.join("studyfrog.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let line = format!("calibrated_start = {on}");
    std::fs::write(
        &path,
        with_setting_line(&text, "calibrated_start", line, Some("srs")),
    )
    .map_err(|e| ServiceError::Io(format!("{}: {e}", path.display())))?;
    studyfrog_store::reindex::reindex(&mut session.conn, &session.root)
        .map_err(ServiceError::Store)?;
    vault_info(&session.root, &session.conn)
}
