//! The `status` front-matter key on every kind of document.

use studyfrog_core::status::{Status, status_from_name, status_name};

use crate::doc::Doc;
use crate::extra::{extra_value, with_extra};

/// The stored status; `None` when the key is absent or holds a word that is not a status (the text stays in the file).
pub fn status_of(doc: &Doc) -> Option<Status> {
    extra_value(doc, "status").and_then(|w| status_from_name(&w))
}

/// The document with the status set, or the key removed for `None`.
pub fn with_status(doc: &Doc, status: Option<Status>) -> Doc {
    with_extra(doc, "status", status.map(status_name))
}
