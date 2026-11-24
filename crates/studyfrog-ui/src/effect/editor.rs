//! Document and link effects.

use {studyfrog_core::uid::Uid, studyfrog_md::doc::Doc};

/// Document and link effects.
#[derive(Debug, Clone, PartialEq)]
// The largest payload is a rare save or import outcome; boxing it would change every call site.
#[allow(clippy::large_enum_variant)]
pub enum EffectEditor {
    /// Read the link graph from the index.
    LoadGraph,
    /// Read a document for editing.
    LoadDocument(Uid),
    /// Write an edited document, if the file is unchanged since `hash`.
    #[rustfmt::skip]
    SaveDocument { #[doc = "Vault path."] path: String, #[doc = "Doc."] doc: Doc, #[doc = "Hash."] hash: u64 },
    /// Find the item a wikilink points to.
    ResolveLink(String),
}
