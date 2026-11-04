//! Data types, options, progress reports, and errors for PDF export.

use std::fmt;
use std::path::PathBuf;

use studyfrog_core::uid::Uid;
use studyfrog_pdf::write::{Margins, PageSize, default_margins};
use studyfrog_store::error::StoreError;

use crate::session::ServiceError;

/// Which items to include in the PDF export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportTarget {
    /// A single note, card, or question.
    SingleItem(Uid),
    /// A user-chosen list of items.
    Selection(Vec<Uid>),
    /// A stack folder (e.g. `""` for root or `"science/physics"`), including sub-stacks and referenced items.
    Stack(String),
}

/// User-configurable options for the generated PDF document.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfExportOptions {
    /// ISO A4, US Letter, or custom page dimensions.
    pub page_size: PageSize,
    /// Inset margins in points.
    pub margins: Margins,
    /// Base font size in points for body paragraphs.
    pub font_size: f32,
    /// Whether to prepend a cover/title page with stack/document metadata.
    pub title_page: bool,
    /// Whether to generate an interactive Table of Contents.
    pub table_of_contents: bool,
    /// Whether to print running page numbers in the footer.
    pub page_numbers: bool,
    /// Whether to include back answers of flashcards and reference answers of questions.
    pub show_answers: bool,
}

/// Default PDF export options (A4, standard margins, 10 pt font, answers shown, no title page or TOC).
pub fn default_pdf_export_options() -> PdfExportOptions {
    PdfExportOptions {
        page_size: PageSize::A4,
        margins: default_margins(),
        font_size: 10.0,
        title_page: false,
        table_of_contents: false,
        page_numbers: true,
        show_answers: true,
    }
}

/// Cooperative progress update reported during an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportProgress {
    /// Number of items processed so far.
    pub current: usize,
    /// Total number of items to process.
    pub total: usize,
}

/// Outcome of a successful PDF export operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfExportOutcome {
    /// Where the PDF was saved.
    pub destination: PathBuf,
    /// Size of the written file in bytes.
    pub bytes_written: usize,
    /// Number of pages in the output document.
    pub pages: usize,
    /// Count of items included in the PDF.
    pub item_count: usize,
    /// Non-fatal warnings encountered (e.g. missing vault pictures).
    pub warnings: Vec<String>,
}

/// Reasons why a PDF export cannot proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfExportError {
    /// The specified file path or name is hostile or forbidden.
    HostilePath(String),
    /// The operation was cancelled cooperatively by the user.
    Cancelled,
    /// The target scope contains zero valid items.
    EmptyExport,
    /// A requested item or stack was not found in the vault.
    NotFound(String),
    /// An I/O error occurred while reading or writing.
    Io(String),
    /// A database query error occurred.
    Store(String),
    /// The PDF layout or serialization failed.
    Pdf(String),
}

impl fmt::Display for PdfExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HostilePath(p) => write!(f, "refused hostile or invalid export path: {p}"),
            Self::Cancelled => write!(f, "PDF export was cancelled"),
            Self::EmptyExport => write!(f, "nothing to export"),
            Self::NotFound(item) => write!(f, "not found: {item}"),
            Self::Io(e) => write!(f, "I/O error during PDF export: {e}"),
            Self::Store(e) => write!(f, "store error during PDF export: {e}"),
            Self::Pdf(e) => write!(f, "PDF layout error: {e}"),
        }
    }
}

impl std::error::Error for PdfExportError {}

impl From<PdfExportError> for ServiceError {
    fn from(err: PdfExportError) -> Self {
        match err {
            PdfExportError::HostilePath(msg) => {
                ServiceError::Invalid(format!("hostile path: {msg}"))
            }
            PdfExportError::Cancelled => ServiceError::Invalid("PDF export cancelled".to_string()),
            PdfExportError::EmptyExport => ServiceError::Invalid("nothing to export".to_string()),
            PdfExportError::NotFound(msg) => ServiceError::Invalid(msg),
            PdfExportError::Io(msg) => ServiceError::Io(msg),
            PdfExportError::Store(msg) => ServiceError::Store(StoreError::Sql(msg)),
            PdfExportError::Pdf(msg) => ServiceError::Invalid(msg),
        }
    }
}
