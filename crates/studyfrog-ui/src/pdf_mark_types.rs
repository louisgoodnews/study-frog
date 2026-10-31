//! Messages about the highlights and notes of the open PDF.

use studyfrog_config::pdf_marks::PdfMark;

/// The marks of a PDF and where each is now (`None`: the text it covered is gone).
pub type MarkSet = (Vec<PdfMark>, Vec<Option<usize>>);

/// A change to the marks of the open PDF.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkMsg {
    /// The marks of a file arrived.
    Loaded(String, Result<Box<MarkSet>, String>),
    /// The marks were written, or could not be.
    Saved(Result<(), String>),
    /// Highlight the selected words in the chosen colour.
    Highlight,
    /// Choose the colour for new marks.
    Color(String),
    /// The next click on the page places a note.
    ToggleNoteMode,
    /// A note was placed at this point of the shown page, in points from the top left.
    PlaceNote(f32, f32),
    /// The note text of a mark changed.
    EditNote(u32, String),
    /// Remove a mark.
    Delete(u32),
    /// Go to the page a mark is on.
    Jump(u32),
}
