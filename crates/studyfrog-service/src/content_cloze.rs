//! What a run shows for an item with cloze gaps (STORY-0108, `docs/specs/cloze.md`): the gaps of one group
//! hidden in the prompt, highlighted in the answer, and their words as what a typed answer is compared with.

use studyfrog_md::cloze::{active_group, cloze_answers, cloze_groups, mask_cloze, reveal_cloze};
use studyfrog_md::doc::Doc;
use studyfrog_md::extra::extra_value;

use crate::content::ItemContent;

/// The front-matter key that makes a card always ask one group.
pub const GROUP_FIELD: &str = "cloze_group";

/// The text with gaps and the extra text shown after the answer, for a card or note that has gaps.
fn gapped(doc: &Doc) -> Option<(&str, &str)> {
    let (text, extra) = match doc {
        Doc::Card(c) => (c.front.as_str(), c.back.as_str()),
        Doc::Note(n) => (n.body.as_str(), ""),
        _ => return None,
    };
    (!cloze_groups(text).is_empty()).then_some((text, extra))
}

/// The group a run asks of this document after `reviews` reviews, when it has gaps.
pub fn asked_group(doc: &Doc, reviews: usize) -> Option<u8> {
    let (text, _) = gapped(doc)?;
    let fixed = extra_value(doc, GROUP_FIELD).and_then(|g| g.parse().ok());
    active_group(&cloze_groups(text), reviews, fixed)
}

/// The words of `group` a typed answer is compared with.
pub fn expected_words(doc: &Doc, group: u8) -> Option<String> {
    let (text, _) = gapped(doc)?;
    Some(cloze_answers(text, group).join(", "))
}

/// `content` turned into its cloze form after `reviews` reviews; unchanged without gaps.
pub fn with_cloze(doc: &Doc, content: ItemContent, reviews: usize) -> ItemContent {
    let (Some((text, extra)), Some(group)) = (gapped(doc), asked_group(doc, reviews)) else {
        return content;
    };
    let text = crate::content::unescape_newlines(text.trim_end());
    let revealed = reveal_cloze(&text, group);
    let answer = match extra.trim() {
        "" => revealed,
        more => format!("{revealed}\n\n{}", crate::content::unescape_newlines(more)),
    };
    ItemContent {
        prompt: mask_cloze(&text, group),
        answer,
        expected: Some(cloze_answers(&text, group).join(", ")),
        ..content
    }
}
