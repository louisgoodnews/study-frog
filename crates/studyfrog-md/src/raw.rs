//! Serde mirror of the front-matter keys. Field order is the canonical output order.

use serde::{Deserialize, Serialize};

/// `srs:` overrides. Unknown override keys are an error, never silently dropped.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSrs {
    /// Target retention override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_retention: Option<f32>,
}

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
    /// Stack daily new limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_new_limit: Option<u32>,
    /// Stack daily review limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_review_limit: Option<u32>,
    /// Stack SRS overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub srs: Option<RawSrs>,
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
