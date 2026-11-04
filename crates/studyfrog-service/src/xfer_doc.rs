//! The common export document structure shared across JSON, TOML and RON (TASK-0220).

use serde::{Deserialize, Serialize};

/// The format identifier for StudyFrog exports.
pub const FORMAT: &str = "studyfrog-export";
/// Current export version.
pub const VERSION: u64 = 1;

/// One stack in an export document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportStack {
    /// Folder relative to vault.
    pub folder: String,
    /// Stack title.
    pub title: String,
    /// Optional description.
    #[serde(default)]
    pub description: String,
}

/// One review in an item's history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportReview {
    /// When the review took place in Unix milliseconds.
    pub at: i64,
    /// Rating word: again, hard, good, easy.
    pub rating: String,
    /// Days elapsed since the previous review.
    #[serde(default)]
    pub elapsed_days: f32,
}

/// Per-stack memory values when exported with scheduling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportStackMemory {
    /// Stack folder or "Inbox".
    pub stack: String,
    /// Source: "own" or "referenced".
    pub source: String,
    /// FSRS difficulty in this stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<f32>,
    /// Familiarity value (0.0 to 1.0) in this stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub familiarity: Option<f32>,
    /// Next due timestamp in Unix milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<i64>,
}

/// Overall average memory values for an item across its referencing stacks.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ExportAverage {
    /// Average familiarity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub familiarity: Option<f32>,
    /// Average difficulty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<f32>,
}

/// One item in an export document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportItem {
    /// Note, card or question.
    pub kind: String,
    /// Item title.
    #[serde(default)]
    pub title: String,
    /// Primary stack folder.
    pub stack: String,
    /// Front or note body.
    pub text: String,
    /// Back or reference answer.
    #[serde(default)]
    pub answer: String,
    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Other stacks referencing this item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_in: Vec<String>,
    /// Unique item ULID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    /// Stated difficulty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    /// Average values across referencing stacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub average: Option<ExportAverage>,
    /// Per-stack memory values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stacks: Vec<ExportStackMemory>,
    /// Review history.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reviews: Vec<ExportReview>,
}

/// Top-level interchange document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportDoc {
    /// Format identifier.
    pub format: String,
    /// Format version.
    pub version: u64,
    /// ISO 8601 timestamp when exported.
    #[serde(default)]
    pub exported: String,
    /// Whether review histories and per-stack values are included.
    #[serde(default)]
    pub scheduling: bool,
    /// Stacks in the export.
    #[serde(default)]
    pub stacks: Vec<ExportStack>,
    /// Items in the export.
    #[serde(default)]
    pub items: Vec<ExportItem>,
}
