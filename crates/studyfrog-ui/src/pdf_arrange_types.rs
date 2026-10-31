//! Rearranging a PDF into a new file: the plan being edited and what can be done to it.

use studyfrog_pdf::edit::PageSpec;
use studyfrog_service::pdf_ops::SavedCopy;

/// The editable plan of a new file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Arrange {
    /// The files pages come from; the first is the open PDF.
    pub sources: Vec<String>,
    /// The pages of the new file, in order.
    pub plan: Vec<PageSpec>,
    /// Earlier plans, newest last, for undo.
    pub undo: Vec<Vec<PageSpec>>,
    /// The page (position in the plan) that rotate, delete and move act on.
    pub selected: Option<usize>,
    /// Vault-relative path of the new file.
    pub name: String,
    /// The page ranges to split into, as typed.
    pub ranges: String,
    /// Write the highlights and notes into the copy.
    pub include_marks: bool,
    /// Title, author and subject as typed.
    pub props: [String; 3],
    /// Something is being written.
    pub busy: bool,
    /// What happened last, or what is wrong.
    pub message: Option<String>,
}

/// Which property is being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    /// Title.
    Title,
    /// Author.
    Author,
    /// Subject.
    Subject,
}

/// A change to the plan or a request to write it.
#[derive(Debug, Clone, PartialEq)]
pub enum ArrangeMsg {
    /// Start rearranging the open PDF.
    Start,
    /// Leave without writing.
    Cancel,
    /// Choose a page of the plan.
    Select(usize),
    /// Turn the chosen page by 90 (clockwise) or -90 degrees.
    Rotate(i32),
    /// Take the chosen page out.
    Delete,
    /// Move the chosen page earlier (-1) or later (1).
    Move(i32),
    /// Undo the last change.
    Undo,
    /// Back to the original pages.
    Reset,
    /// Ask for another PDF to append.
    AddFile,
    /// The chosen file (absolute path) and its page count, nothing if cancelled.
    FileAdded(Result<Option<(String, usize)>, String>),
    /// The name of the new file.
    Name(String),
    /// The page ranges to split into.
    Ranges(String),
    /// Include the marks.
    IncludeMarks(bool),
    /// A property field.
    Property(Prop, String),
    /// Write the new file.
    Save,
    /// The new file was written, or not.
    Saved(Result<SavedCopy, String>),
    /// Write one file per range.
    Split,
    /// The split files were written, or not.
    Splitted(Result<Vec<String>, String>),
}
