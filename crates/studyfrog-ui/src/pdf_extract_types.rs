//! Making notes from the text of a PDF.

use studyfrog_service::session::VaultInfo;
use studyfrog_store::pdf_notes::SourcedItem;

/// The "make a note" form.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtractForm {
    /// The stack (vault-relative folder) the note goes into.
    pub folder: String,
    /// Last page to take (0-based); the first is the page being read.
    pub last: Option<usize>,
    /// Use the selected words instead of the pages.
    pub use_selection: bool,
    /// Title; empty takes a name from the PDF.
    pub title: String,
    /// Something is being made.
    pub busy: bool,
    /// Pictures saved from the PDF, as vault-relative paths.
    pub pictures: Vec<String>,
    /// Put the saved pictures at the end of the note.
    pub include_pictures: bool,
    /// The next dragged rectangle is saved as a picture.
    pub snapshot_mode: bool,
}

/// What a finished note gives back: the vault as it is now, and the note's path.
pub type Made = Result<Box<(VaultInfo, String)>, String>;

/// A change to the form or a request.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractMsg {
    /// The stack changed.
    Folder(String),
    /// The last page changed (0-based).
    Last(usize),
    /// Selection or pages.
    UseSelection(bool),
    /// The title changed.
    Title(String),
    /// Save the pictures on the shown page.
    SavePictures,
    /// The pictures were saved, or not.
    PicturesSaved(Result<Vec<String>, String>),
    /// The next dragged rectangle becomes a picture.
    ToggleSnapshot,
    /// A rectangle of the shown page was dragged (points from the top left).
    Snapshot(studyfrog_pdf::render::PageRect),
    /// The snapshot was saved, or not.
    SnapshotSaved(Result<String, String>),
    /// Put the saved pictures into the note.
    IncludePictures(bool),
    /// Make the note.
    Make,
    /// The note was made, or not.
    Made(Made),
    /// The items made from a PDF arrived.
    Sourced(String, Result<Vec<SourcedItem>, String>),
}
