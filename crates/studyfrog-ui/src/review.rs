//! The post-run review: per item, the previous rating, the user's own mark and rating, and the
//! rating suggested by reading speed. Pure; computed from the event log and the loaded content.

use studyfrog_core::event::{Difficulty, EventKind};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::reading::{
    Suggestion, item_records, suggest, suggest_from_mark, word_count,
};
use studyfrog_slm::score::Verdict;

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
    /// The rating the item's reading time and the user's mark suggest instead.
    pub suggestion: Option<Suggestion>,
    /// How the typed answer compared with the reference, as a score from 0 to 100 and its verdict.
    pub answer_match: Option<(u32, Verdict)>,
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
            let by_reading = match (r.read_ms, r.rated) {
                (Some(ms), Some(rated)) => suggest(words, ms, previous, rated, &params),
                _ => None,
            };
            let by_mark = r.rated.and_then(|rated| suggest_from_mark(r.marked, rated));
            let suggestion = by_mark.or(by_reading);
            ReviewRow {
                item: r.item,
                title: content.map_or_else(String::new, |c| c.title.clone()),
                previous,
                marked: r.marked,
                rated: r.rated,
                reading: r.read_ms.map(|ms| (words, ms / 100)),
                suggestion,
                answer_match: run
                    .scores
                    .get(&r.item)
                    .map(|s| ((s.score.value * 100.0).round() as u32, s.score.verdict)),
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
