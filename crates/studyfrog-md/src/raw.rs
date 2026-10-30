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
}
