//! The post-run review: per item, the previous rating, the user's own mark and rating, and the
//! rating suggested by reading speed. Pure; computed from the event log and the loaded content.

use studyfrog_core::event::{Difficulty, EventKind};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::reading::{Suggestion, item_records, suggest, word_count};

use crate::run_state::RunView;

/// One line of the review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    /// The item.
    pub item: Uid,
    /// Its title.
    pub title: String,
    /// The rating of its previous review (`None`: never reviewed before this run).
    pub previous: Option<Rating>,
    /// What the user marked while it was on screen.
    pub marked: Option<Difficulty>,
    /// The rating given in this run.
    pub rated: Option<Rating>,
    /// Words on the back and seconds spent reading them, in tenths.
    pub reading: Option<(usize, u32)>,
    /// The rating reading speed suggests instead.
    pub suggestion: Option<Suggestion>,
}

/// Items that were rated or marked, in the order they first appeared.
pub fn review_rows(run: &RunView) -> Vec<ReviewRow> {
    let params = run.reading;
    item_records(&run.events)
        .into_iter()
        .filter(|r| r.rated.is_some() || r.marked.is_some())
        .map(|r| {
            let content = run.content.get(&r.item);
            let previous = content.and_then(|c| c.previous);
            let words = content.map_or(0, |c| word_count(&c.answer));
            let suggestion = match (r.read_ms, r.rated) {
                (Some(ms), Some(rated)) => suggest(words, ms, previous, rated, &params),
                _ => None,
            };
            ReviewRow {
                item: r.item,
                title: content.map_or_else(String::new, |c| c.title.clone()),
                previous,
                marked: r.marked,
                rated: r.rated,
                reading: r.read_ms.map(|ms| (words, ms / 100)),
                suggestion,
            }
        })
        .collect()
}

/// The mark given to the item now on screen (the last one since it was shown).
pub fn current_mark(run: &RunView) -> Option<Difficulty> {
    let item = studyfrog_rehearsal::session::current_item(&run.session)?;
    run.events
        .iter()
        .rev()
        .filter(|e| e.item == Some(item))
        .find_map(|e| match &e.kind {
            EventKind::ItemMarked(d) => Some(Some(*d)),
            EventKind::ItemShown => Some(None),
            _ => None,
        })
        .flatten()
}
