//! Diary entries: self-reported reflections, ratings, and linked entities.
//! See `docs/specs/diary.md`.

use std::collections::BTreeMap;

use crate::civil::{Date, format_date};
use crate::time::UnixMs;
use crate::uid::Uid;

/// The 13 standard self-assessment rating scale identifiers.
pub const STANDARD_SCALES: [&str; 13] = [
    "performance",
    "success",
    "confidence",
    "effort",
    "focus",
    "energy",
    "difficulty",
    "satisfaction",
    "understanding",
    "mood",
    "stress",
    "motivation",
    "sleep",
];

/// A rating value in a diary entry: either an integer score or a string label.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RatingValue {
    /// Numeric rating (e.g. 1..=5 for standard scales, or custom scale score).
    Int(i64),
    /// Text label (e.g. custom categorical rating).
    Text(String),
}

/// A diary entry data model.
#[derive(Debug, Clone, PartialEq)]
pub struct DiaryEntry {
    /// Unique stable identifier (canonical ULID).
    pub uid: Uid,
    /// Civil date of the diary entry.
    pub date: Date,
    /// Optional title heading; defaults to civil date display.
    pub title: Option<String>,
    /// When the entry was created.
    pub created: Option<UnixMs>,
    /// When the entry was last edited.
    pub updated: Option<UnixMs>,
    /// Study time in minutes.
    pub study_minutes: Option<u32>,
    /// Tags attached to the entry.
    pub tags: Vec<String>,
    /// Ratings mapped by scale key.
    pub ratings: BTreeMap<String, RatingValue>,
    /// Rehearsal runs linked to this entry.
    pub runs: Vec<Uid>,
    /// Stacks studied or referenced (ULID strings or folder paths).
    pub stacks: Vec<String>,
    /// Specific cards, notes, or questions referenced.
    pub items: Vec<Uid>,
    /// Calendar exam or session IDs referenced.
    pub exams: Vec<String>,
    /// Markdown body containing reflections.
    pub content: String,
}

/// Whether `name` matches one of the 13 standard rating scales.
pub fn is_standard_scale(name: &str) -> bool {
    STANDARD_SCALES.contains(&name)
}

/// The effective display title of a diary entry: explicit title or "Diary: YYYY-MM-DD".
pub fn diary_title(entry: &DiaryEntry) -> String {
    entry
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .map_or_else(
            || format!("Diary: {}", format_date(entry.date)),
            ToOwned::to_owned,
        )
}

/// Extract an integer score from a [`RatingValue`].
pub fn rating_int(value: &RatingValue) -> Option<i64> {
    match value {
        RatingValue::Int(v) => Some(*v),
        RatingValue::Text(_) => None,
    }
}

/// Extract a 1..=5 standard score from a [`RatingValue`].
pub fn rating_standard_score(value: &RatingValue) -> Option<u8> {
    match value {
        RatingValue::Int(v) if (1..=5).contains(v) => Some(*v as u8),
        _ => None,
    }
}
