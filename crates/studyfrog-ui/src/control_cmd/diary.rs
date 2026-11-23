//! The diary commands of the control protocol: the entries of a day or a range, one entry opened in
//! the editor, and a new entry saved.

use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;

/// What `diary ...` asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum DiaryCmd {
    /// The entries from `from` to `to` (both inclusive; `to` missing: the day `from`; both missing:
    /// all entries).
    List {
        /// The first day.
        from: Option<Date>,
        /// The last day.
        to: Option<Date>,
    },
    /// Open the entry with this ULID in the editor and answer with it.
    Open(Uid),
    /// Write and save a new entry.
    Add(DiaryNew),
}

/// The fields of a new diary entry.
#[derive(Debug, Clone, PartialEq)]
pub struct DiaryNew {
    /// The day of the entry.
    pub date: Date,
    /// The title.
    pub title: String,
    /// The Markdown body.
    pub text: String,
    /// Study time in minutes.
    pub minutes: Option<u32>,
    /// Tags, comma separated.
    pub tags: String,
    /// Stacks, comma separated.
    pub stacks: String,
}
