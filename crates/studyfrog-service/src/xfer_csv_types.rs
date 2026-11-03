//! The types of reading CSV with a column mapping, and the words about its problems.

use crate::xfer_model::Bundle;

/// Which column means what (column numbers start at 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mapping {
    /// The front of a card, a question's prompt, a note's body.
    pub front: usize,
    /// The back of a card, a question's answer.
    pub back: usize,
    /// The title.
    pub title: Option<usize>,
    /// The tags, separated by semicolons.
    pub tags: Option<usize>,
    /// The stack folder.
    pub stack: Option<usize>,
    /// The kind: card, question or note.
    pub kind: Option<usize>,
}

/// What the user chose; every `None` is detected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CsvOptions {
    /// The delimiter.
    pub delimiter: Option<char>,
    /// Read the bytes as Latin-1 (`true`) or UTF-8 (`false`).
    pub latin1: Option<bool>,
    /// The first row names the columns.
    pub header: Option<bool>,
    /// The meaning of the columns.
    pub map: Option<Mapping>,
}

/// What was used, for the dialog to show and change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    /// The delimiter used.
    pub delimiter: char,
    /// The bytes were read as Latin-1.
    pub latin1: bool,
    /// The first row is a header.
    pub header: bool,
    /// The column names (or "Column 1", ...).
    pub names: Vec<String>,
    /// The first rows after the header.
    pub first_rows: Vec<Vec<String>>,
    /// The mapping used.
    pub map: Mapping,
}

/// One thing wrong in the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The row as a spreadsheet numbers it (the header is row 1).
    pub row: usize,
    /// The column's name.
    pub column: String,
    /// What was expected there.
    pub expected: String,
    /// What was found.
    pub found: String,
}

/// The result of reading a file.
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRead {
    /// The rows without problems.
    pub bundle: Bundle,
    /// How the file was read.
    pub sheet: Sheet,
    /// Every problem.
    pub problems: Vec<Problem>,
}

/// A problem in words.
pub fn problem_text(p: &Problem) -> String {
    let found = if p.found.is_empty() {
        "nothing".to_owned()
    } else {
        format!("\"{}\"", p.found)
    };
    format!(
        "row {}, column {}: expected {}, found {found}",
        p.row, p.column, p.expected
    )
}

/// A message to hand to an AI with the file: what the layout is and exactly which rows to fix.
pub fn ai_message(layout: &str, problems: &[Problem]) -> String {
    let list: Vec<String> = problems
        .iter()
        .map(|p| format!("- {}", problem_text(p)))
        .collect();
    format!(
        "Please fix my CSV file for StudyFrog. Change only the rows named below and send back the whole file as CSV text.\n\nThe layout StudyFrog expects: {layout}\n\nProblems found:\n{}\n",
        list.join("\n")
    )
}
