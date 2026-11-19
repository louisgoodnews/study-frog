//! PDF ingestion through the unified import hub target model.

use std::path::Path;

use crate::create::{IdSource, refresh};
use crate::pdf_library::add_pdf;
use crate::pdfs::index_pdfs_fully;
use crate::session::{SETTLE_NS, ServiceError, VaultInfo, VaultSession, vault_info};
use crate::xfer_model::ImportTarget;
use crate::xfer_target::ensure_new_stack;

/// Add a PDF at `source` to an existing or newly created stack and index it immediately.
pub fn import_pdf_to(
    session: &mut VaultSession,
    source: &Path,
    target: &ImportTarget,
    ids: IdSource,
    now_ns: i64,
) -> Result<(VaultInfo, String), ServiceError> {
    let settle_now = now_ns.saturating_add(SETTLE_NS);
    match target {
        ImportTarget::ExistingStack(folder) => {
            let rel = add_pdf(session, source, folder)?;
            index_pdfs_fully(session, settle_now)?;
            Ok((vault_info(&session.root, &session.conn)?, rel))
        }
        ImportTarget::NewStack(name) => {
            let folder = ensure_new_stack(session, name, ids)?;
            let rel = add_pdf(session, source, &folder)?;
            let _ = refresh(session)?;
            index_pdfs_fully(session, settle_now)?;
            Ok((vault_info(&session.root, &session.conn)?, rel))
        }
        ImportTarget::VaultRootNotes | ImportTarget::FromSource => Err(ServiceError::Invalid(
            "choose an existing or new stack for PDF files".to_owned(),
        )),
    }
}
