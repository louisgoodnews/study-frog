//! Work the PDF viewer asks the worker for.

use crate::pdf_types::PageKey;

/// Work the viewer asks the worker for.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfEffect {
    /// Open a PDF of the vault and read its structure.
    Open(String),
    /// Draw one page of a PDF.
    Render {
        /// Vault-relative path.
        path: String,
        /// Which page at what scale.
        key: PageKey,
    },
    /// Read the words of one page (0-based).
    Text {
        /// Vault-relative path.
        path: String,
        /// The page.
        page: usize,
    },
    /// Find the pages of a PDF that contain a text.
    Find {
        /// Vault-relative path.
        path: String,
        /// The text.
        query: String,
    },
    /// Find PDFs by title and text for the global search.
    Search(String),
    /// Ask for another PDF to merge in.
    PickMerge,
    /// Write a new PDF from a plan.
    SaveCopy(studyfrog_service::pdf_ops::SaveRequest),
    /// Write one file per page range.
    Split {
        /// Vault-relative path of the PDF.
        path: String,
        /// 0-based first and last page of each range.
        ranges: Vec<(usize, usize)>,
    },
    /// Make a note from pages or a selection.
    MakeNote {
        /// Vault-relative path of the PDF.
        path: String,
        /// First page (0-based).
        first: usize,
        /// Last page (0-based).
        last: usize,
        /// The selected text, if that is what to use.
        selection: Option<String>,
        /// The stack folder.
        folder: String,
        /// The title, or empty.
        title: String,
        /// Pictures to put at the end.
        pictures: Vec<String>,
    },
    /// Save the pictures of a page.
    SavePictures {
        /// Vault-relative path.
        path: String,
        /// The page (0-based).
        page: usize,
    },
    /// Save a rectangle of a page as a picture.
    SaveRegion {
        /// Vault-relative path.
        path: String,
        /// The page (0-based).
        page: usize,
        /// The rectangle.
        rect: studyfrog_pdf::render::PageRect,
    },
    /// Adding, listing and renaming PDFs.
    Library(crate::pdf_library_types::LibraryJob),
    /// Make a card or a question from a passage.
    MakeItem(crate::pdf_draft_types::ItemJob),
    /// List the items made from a PDF.
    LoadSourced(String),
    /// List the pages of a PDF that have no text.
    LoadScanned(String),
    /// Read the highlights and notes of a PDF.
    LoadMarks(String),
    /// Write the highlights and notes of a PDF.
    SaveMarks {
        /// Vault-relative path.
        path: String,
        /// All of its marks.
        marks: Vec<studyfrog_config::pdf_marks::PdfMark>,
    },
    /// Exporting items or stacks to PDF.
    Export(crate::pdf_export_types::PdfExportJob),
}
