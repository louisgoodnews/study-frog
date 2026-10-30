//! Stack: a container of items (the `_stack.md` definition).

use crate::meta::Meta;

/// A stack definition.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// Common fields.
    pub meta: Meta,
    /// The `description` key; empty when absent.
    pub description: String,
    /// The `daily_new_limit` key.
    pub daily_new_limit: Option<u32>,
    /// The `daily_review_limit` key.
    pub daily_review_limit: Option<u32>,
    /// Per-stack override of the SRS target retention (`srs.request_retention`).
    /// Other SRS overrides are added with the SRS parameter type (TASK-0016).
    pub request_retention: Option<f32>,
}
