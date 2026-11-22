//! The case study a question item is, for the rehearsal screen (STORY-0199). Pure.

use studyfrog_md::doc::Doc;

use crate::case_codec::case_from_doc;
use crate::content::ItemContent;

/// The content with the case study filled in, when the question is one; a malformed case is left out
/// (the item still shows as a plain question).
pub fn with_case(doc: &Doc, content: ItemContent) -> ItemContent {
    match case_from_doc(doc) {
        Ok(Some(case)) => ItemContent {
            case: Some(Box::new(case)),
            ..content
        },
        _ => content,
    }
}
