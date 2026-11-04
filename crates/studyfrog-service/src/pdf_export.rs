//! PDF export use cases: exports notes, cards, questions, selections, or entire stacks to PDF.

use std::path::Path;

pub use crate::pdf_export_exec::execute_pdf_export;
pub use crate::pdf_export_plan::{PdfExportPlan, plan_pdf_export};
pub use crate::pdf_export_scope::{TargetScope, resolve_target_scope};
pub use crate::pdf_export_types::{
    ExportProgress, ExportTarget, PdfExportError, PdfExportOptions, PdfExportOutcome,
    default_pdf_export_options,
};

use crate::session::{ServiceError, VaultSession};

/// Export the given target to a destination PDF file.
pub fn export_pdf(
    session: &VaultSession,
    target: &ExportTarget,
    options: &PdfExportOptions,
    destination: &Path,
) -> Result<PdfExportOutcome, ServiceError> {
    execute_pdf_export(session, target, options, destination, None, None)
        .map_err(ServiceError::from)
}
