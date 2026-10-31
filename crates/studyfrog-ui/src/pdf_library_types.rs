//! The PDFs of the vault as the library lists them.

use studyfrog_store::pdf_index::PdfRow;

/// The indexed PDFs and a rename being typed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfLibrary {
    /// Every indexed PDF, by title.
    pub rows: Vec<PdfRow>,
    /// The PDF being renamed (path) and the name typed so far.
    pub rename: Option<(String, String)>,
}

/// Work for the worker.
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryJob {
    /// List the indexed PDFs.
    Load,
    /// Ask for a PDF and copy it into a stack.
    Add {
        /// The stack folder.
        folder: String,
    },
    /// Give a PDF a new file name.
    Rename {
        /// Vault-relative path.
        path: String,
        /// The new file name.
        name: String,
    },
}

/// Changes to the library's PDF list.
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryMsg {
    /// Ask for the list.
    Load,
    /// The list arrived.
    Loaded(Result<Vec<PdfRow>, String>),
    /// Add a PDF to this stack (the empty folder means none is chosen).
    AddTo(String),
    /// A PDF was added (its path), nothing if cancelled, or it failed.
    Added(Result<Option<String>, String>),
    /// Start renaming a PDF.
    StartRename(String),
    /// The new name changed.
    RenameEdit(String),
    /// Do the rename.
    DoRename,
    /// Stop renaming.
    CancelRename,
    /// The rename finished: the new path and how many files were rewritten.
    Renamed(Result<(String, usize), String>),
}
