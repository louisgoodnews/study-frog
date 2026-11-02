//! The columns of the library list: their words and headings, which can be sorted, what a cell shows and how the choice is stored.

use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_core::plan::item_kind_name;
use studyfrog_core::priority::priority_name;
use studyfrog_core::query::Field;
use studyfrog_core::query_eval::Row;
use studyfrog_core::status::status_label;

/// A column of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    /// The title (always shown).
    Title,
    /// Note, card or question.
    Type,
    /// The status.
    Status,
    /// The priority.
    Priority,
    /// The difficulty.
    Difficulty,
    /// Familiarity as a percentage.
    Familiarity,
    /// The stack.
    Stack,
    /// The subjects.
    Subjects,
    /// The tags.
    Tags,
    /// How many reviews.
    Reviews,
    /// The next review.
    Due,
    /// When it was created.
    Created,
    /// When it was last updated.
    Updated,
    /// The file's path.
    Path,
}

/// Every column, with its stored word and its heading.
pub const COLUMNS: [(Column, &str, &str); 14] = [
    (Column::Title, "title", "Title"),
    (Column::Type, "type", "Type"),
    (Column::Status, "status", "Status"),
    (Column::Priority, "priority", "Priority"),
    (Column::Difficulty, "difficulty", "Difficulty"),
    (Column::Familiarity, "familiarity", "Familiarity"),
    (Column::Stack, "stack", "Stack"),
    (Column::Subjects, "subjects", "Subjects"),
    (Column::Tags, "tags", "Tags"),
    (Column::Reviews, "reviews", "Reviews"),
    (Column::Due, "due", "Due"),
    (Column::Created, "created", "Created"),
    (Column::Updated, "updated", "Updated"),
    (Column::Path, "path", "Path"),
];

/// The columns shown until the user chooses others.
pub const DEFAULT_COLUMNS: [Column; 5] = [
    Column::Title,
    Column::Type,
    Column::Status,
    Column::Priority,
    Column::Due,
];

/// The heading of a column.
pub fn column_label(c: Column) -> &'static str {
    COLUMNS
        .iter()
        .find(|(x, _, _)| *x == c)
        .map_or("", |(_, _, l)| l)
}

/// The columns named in a stored list (`title,type,status`); unknown words are ignored and the title is always there.
pub fn columns_from_text(text: &str) -> Vec<Column> {
    let mut out = vec![Column::Title];
    for word in text.split(',').map(str::trim) {
        let found = COLUMNS
            .iter()
            .find(|(_, w, _)| *w == word)
            .map(|(c, _, _)| *c);
        if let Some(c) = found.filter(|c| !out.contains(c)) {
            out.push(c);
        }
    }
    out
}

/// The columns as the text to store, in the order of [`COLUMNS`].
pub fn columns_to_text(columns: &[Column]) -> String {
    let words: Vec<&str> = COLUMNS
        .iter()
        .filter(|(c, _, _)| columns.contains(c))
        .map(|(_, w, _)| *w)
        .collect();
    words.join(",")
}

/// The field a column sorts by; `None` for a column that cannot be sorted.
pub fn column_field(c: Column) -> Option<Field> {
    Some(match c {
        Column::Title => Field::Title,
        Column::Type => Field::Type,
        Column::Status => Field::Status,
        Column::Priority => Field::Priority,
        Column::Difficulty => Field::Difficulty,
        Column::Familiarity => Field::Familiarity,
        Column::Stack => Field::Stack,
        Column::Reviews => Field::Reviews,
        Column::Due => Field::Due,
        Column::Created => Field::Created,
        Column::Updated => Field::Updated,
        Column::Path => Field::Path,
        Column::Subjects | Column::Tags => return None,
    })
}

fn date_text(day: Option<i64>) -> String {
    day.map_or_else(|| "–".to_owned(), |d| format_date(date_from_days(d)))
}

/// What a row shows in a column.
pub fn cell(row: &Row, c: Column) -> String {
    let list = |v: &[String]| {
        if v.is_empty() {
            "–".to_owned()
        } else {
            v.join(", ")
        }
    };
    match c {
        Column::Title => row.title.clone(),
        Column::Type => item_kind_name(row.kind).to_owned(),
        Column::Status => status_label(row.status).to_owned(),
        Column::Priority => priority_name(row.priority).to_owned(),
        Column::Difficulty => row.difficulty.map_or_else(
            || "–".to_owned(),
            |d| studyfrog_core::event::difficulty_name(d).to_owned(),
        ),
        Column::Familiarity => row
            .familiarity
            .map_or_else(|| "–".to_owned(), |f| format!("{:.0} %", f * 100.0)),
        Column::Stack => row.stacks.last().cloned().unwrap_or_else(|| "–".to_owned()),
        Column::Subjects => list(&row.subjects),
        Column::Tags => list(&row.tags),
        Column::Reviews => row.reviews.to_string(),
        Column::Due => date_text(row.due_day),
        Column::Created => date_text(row.created_day),
        Column::Updated => date_text(row.updated_day),
        Column::Path => row.path.clone(),
    }
}
