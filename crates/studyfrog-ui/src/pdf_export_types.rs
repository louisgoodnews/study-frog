//! Data types, messages, jobs, and previews for the PDF export dialog.

use std::path::PathBuf;

use studyfrog_pdf::render::Pixels;
use studyfrog_pdf::write::{Margins, PageSize, default_margins};
use studyfrog_service::pdf_export::{
    ExportTarget, PdfExportOptions, PdfExportOutcome, default_pdf_export_options,
};

/// Normal / standard 54 pt margins.
pub fn margins_normal() -> Margins {
    default_margins()
}

/// Narrow / compact 36 pt margins.
pub fn margins_narrow() -> Margins {
    Margins {
        top: 36.0,
        bottom: 36.0,
        left: 36.0,
        right: 36.0,
    }
}

/// Wide 72 pt margins.
pub fn margins_wide() -> Margins {
    Margins {
        top: 72.0,
        bottom: 72.0,
        left: 72.0,
        right: 72.0,
    }
}

/// Pre-rendered preview of one page of the PDF.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewPage {
    /// 1-based page number.
    pub page_number: usize,
    /// Headings found on this page.
    pub headings: Vec<String>,
    /// First lines of plain text on this page.
    pub snippet: String,
    /// Rendered page bitmap pixels (if rendered).
    pub pixels: Option<Pixels>,
}

/// Preview of a PDF export before writing to disk.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfExportPreview {
    /// Total pages in the document.
    pub total_pages: usize,
    /// Number of items included.
    pub item_count: usize,
    /// Document title.
    pub title: String,
    /// Warnings produced during layout/export.
    pub warnings: Vec<String>,
    /// Previews of the first pages (up to 3).
    pub pages: Vec<PreviewPage>,
}

/// The state of the PDF export dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfExportState {
    /// Target items or stack to export.
    pub target: ExportTarget,
    /// User-configurable options.
    pub options: PdfExportOptions,
    /// Destination path typed or picked.
    pub destination: String,
    /// Generated preview.
    pub preview: Option<PdfExportPreview>,
    /// Status message or details of where file went.
    pub status: Option<String>,
    /// An operation (export or preview) is running.
    pub busy: bool,
    /// Whether preview is currently loading.
    pub previewing: bool,
    /// Error message if last action failed.
    pub error: Option<String>,
}

/// Create a new PDF export dialog state with sensible defaults.
pub fn default_pdf_export_state(target: ExportTarget, destination: String) -> PdfExportState {
    PdfExportState {
        target,
        options: default_pdf_export_options(),
        destination,
        preview: None,
        status: None,
        busy: false,
        previewing: false,
        error: None,
    }
}

fn clean_segment(text: &str) -> String {
    let s: String = text
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    if s.trim_matches('_').is_empty() {
        "export".to_string()
    } else {
        s.trim_matches('_').to_string()
    }
}

/// Compute a friendly default file destination for an export target.
pub fn default_destination(state: &crate::state::State, target: &ExportTarget) -> String {
    let filename = match target {
        ExportTarget::SingleItem(uid) => {
            let title = state
                .library
                .items
                .iter()
                .find(|i| i.uid == *uid)
                .map(|i| i.row.title.as_str())
                .unwrap_or("item");
            format!("{}.pdf", clean_segment(title))
        }
        ExportTarget::Stack(folder) => {
            if folder.trim().is_empty() {
                "library.pdf".to_string()
            } else {
                let seg = folder.rsplit('/').next().unwrap_or(folder);
                format!("{}.pdf", clean_segment(seg))
            }
        }
        ExportTarget::Selection(uids) => {
            format!("selection_{}_items.pdf", uids.len())
        }
    };

    if let Some(vault) = &state.vault {
        std::path::Path::new(&vault.root)
            .join(filename)
            .display()
            .to_string()
    } else {
        filename
    }
}

/// Resolve a destination path, appending extension and joining relative paths to vault root.
pub fn resolve_destination_path(vault: Option<&crate::state::VaultInfo>, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw.trim());
    let path_with_ext = if path.extension().is_none() {
        path.with_extension("pdf")
    } else {
        path
    };
    if let Some(v) = vault.filter(|_| path_with_ext.is_relative()) {
        return std::path::Path::new(&v.root).join(path_with_ext);
    }
    path_with_ext
}

/// What the worker is asked to do for PDF export.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfExportJob {
    /// Generate a preview of the export.
    Preview {
        /// Which items to include.
        target: ExportTarget,
        /// Document options.
        options: PdfExportOptions,
    },
    /// Browse for a destination file.
    Browse,
    /// Write the PDF to destination.
    Export {
        /// Which items to include.
        target: ExportTarget,
        /// Document options.
        options: PdfExportOptions,
        /// Output file destination.
        destination: PathBuf,
    },
}

/// Messages of the PDF export dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfExportMsg {
    /// Open the PDF export dialog for a target.
    Open(ExportTarget),
    /// Set page size.
    SetPageSize(PageSize),
    /// Set margins.
    SetMargins(Margins),
    /// Set font size in points.
    SetFontSize(f32),
    /// Toggle title page.
    SetTitlePage(bool),
    /// Toggle table of contents.
    SetToc(bool),
    /// Toggle page numbers.
    SetPageNumbers(bool),
    /// Toggle answers of cards and questions.
    SetShowAnswers(bool),
    /// Edit the destination path.
    EditDestination(String),
    /// Clicked Browse button to choose file.
    BrowseDestination,
    /// Destination was chosen from picker.
    DestinationPicked(Option<String>),
    /// Preview is ready.
    PreviewReady(Result<PdfExportPreview, String>),
    /// Submit export.
    Submit,
    /// Export completed.
    Exported(Result<PdfExportOutcome, String>),
    /// Close the dialog.
    Close,
}
