//! Fields shared by every item kind (see `docs/specs/markdown-schema.md`).

use crate::time::UnixMs;
use crate::uid::Uid;

/// Common front-matter fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    /// Stable identity from the `id` key.
    pub uid: Uid,
    /// The `title` key.
    pub title: String,
    /// The `tags` key; empty when absent.
    pub tags: Vec<String>,
    /// The `aliases` key: extra wikilink targets; empty when absent.
    pub aliases: Vec<String>,
    /// The `created` key, if present.
    pub created: Option<UnixMs>,
    /// The `updated` key, if present.
    pub updated: Option<UnixMs>,
}
