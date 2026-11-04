//! Serde mirror of the front-matter keys. Field order is the canonical output order.

use serde::{Deserialize, Serialize};

/// All keys any kind understands; unrelated ones stay `None`.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RawFront {
    /// ULID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Kind name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Civil date (YYYY-MM-DD) for diary entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Aliases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    /// RFC 3339 creation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// RFC 3339 update time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    /// Stack description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Note: rehearse the title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rehearse_title: Option<bool>,
    /// Card: also ask back to front.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversible: Option<bool>,
    /// Question: alternative reference answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted: Option<Vec<String>>,
    /// Question: required key points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rubric: Option<Vec<String>>,
    /// Question: pass threshold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,
    /// Diary: study time in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub study_minutes: Option<u32>,
    /// Diary: performance rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance: Option<u8>,
    /// Diary: success rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<u8>,
    /// Diary: confidence rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    /// Diary: effort rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<u8>,
    /// Diary: focus rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<u8>,
    /// Diary: energy rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy: Option<u8>,
    /// Diary: difficulty rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<u8>,
    /// Diary: satisfaction rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub satisfaction: Option<u8>,
    /// Diary: understanding rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub understanding: Option<u8>,
    /// Diary: mood rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mood: Option<u8>,
    /// Diary: stress rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stress: Option<u8>,
    /// Diary: motivation rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motivation: Option<u8>,
    /// Diary: sleep rating.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sleep: Option<u8>,
    /// Diary: linked rehearsal runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs: Option<Vec<String>>,
    /// Diary: linked stacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stacks: Option<Vec<String>>,
    /// Diary: linked items.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<String>>,
    /// Diary: linked exams.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exams: Option<Vec<String>>,
}
