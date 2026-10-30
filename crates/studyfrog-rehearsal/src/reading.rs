//! What the time spent on an item says about it. Pure.
//!
//! People read at a fairly steady pace, so the time between revealing the back of an item and
//! rating it, compared with the time its words should take, hints at how familiar the text is:
//! reading far faster than the baseline means the text was already known (easy) or was not read
//! at all because it means nothing to the reader yet (hard).

use studyfrog_core::event::{Difficulty, EventKind, RunEvent};
use studyfrog_core::review::Rating;
use studyfrog_core::settings::{MAX_READING_WPM, MIN_READING_WPM};
use studyfrog_core::uid::Uid;

/// The assumed reading behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadingParams {
    /// Baseline silent reading speed.
    pub words_per_minute: u32,
    /// Reading in less than this percentage of the expected time counts as fast.
    pub fast_percent: u32,
    /// Reading in more than this percentage of the expected time counts as slow.
    pub slow_percent: u32,
    /// Shorter texts say too little about familiarity.
    pub min_words: usize,
}

/// The speed assumed until the user measures their own: 238 words per minute is the average silent
/// reading speed for non-fiction (Brysbaert, 2019). Fast is under 75 % of the expected time (for 20
/// words, under about 3.8 s) and slow is over 200 % (over about 10 s).
pub fn reading_params_default() -> ReadingParams {
    ReadingParams {
        words_per_minute: DEFAULT_WPM,
        fast_percent: 75,
        slow_percent: 200,
        min_words: 8,
    }
}

/// The default baseline speed.
pub const DEFAULT_WPM: u32 = 238;

/// The defaults with the user's measured speed, when there is one.
pub fn reading_params(measured_wpm: Option<u32>) -> ReadingParams {
    ReadingParams {
        words_per_minute: measured_wpm
            .map_or(DEFAULT_WPM, |w| w.clamp(MIN_READING_WPM, MAX_READING_WPM)),
        ..reading_params_default()
    }
}

/// Words per minute from a timed reading, or `None` when the result is not plausible as reading
/// (nothing read, or outside the accepted speeds).
pub fn measured_wpm(words: usize, ms: u32) -> Option<u32> {
    if words == 0 || ms == 0 {
        return None;
    }
    let wpm = u32::try_from(words as u64 * 60_000 / u64::from(ms)).ok()?;
    (MIN_READING_WPM..=MAX_READING_WPM)
        .contains(&wpm)
        .then_some(wpm)
}

/// Words separated by whitespace.
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// How long `words` words should take to read at the baseline speed.
pub fn expected_ms(words: usize, p: &ReadingParams) -> u32 {
    let ms = words as u64 * 60_000 / u64::from(p.words_per_minute.max(1));
    u32::try_from(ms).unwrap_or(u32::MAX)
}

/// What the log says about one item in a run (the last values win when an item came back).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemRecord {
    /// The item.
    pub item: Uid,
    /// The rating given.
    pub rated: Option<Rating>,
    /// The difficulty the user marked.
    pub marked: Option<Difficulty>,
    /// Milliseconds between revealing the answer and rating it, paused time excluded.
    pub read_ms: Option<u32>,
}

/// One record per item, in order of first appearance.
pub fn item_records(events: &[RunEvent]) -> Vec<ItemRecord> {
    let mut records: Vec<ItemRecord> = Vec::new();
    let mut revealed: Option<(Uid, u32, u32)> = None; // item, t at reveal, pause total at reveal
    let (mut paused_total, mut paused_at) = (0_u32, None::<u32>);
    for e in events {
        match &e.kind {
            EventKind::RunPaused => paused_at = Some(e.t_ms),
            EventKind::RunResumed => {
                paused_total += paused_at.take().map_or(0, |at| e.t_ms.saturating_sub(at));
            }
            _ => {}
        }
        let Some(item) = e.item else { continue };
        let at = match records.iter().position(|r| r.item == item) {
            Some(i) => i,
            None => {
                records.push(ItemRecord {
                    item,
                    rated: None,
                    marked: None,
                    read_ms: None,
                });
                records.len() - 1
            }
        };
        match &e.kind {
            EventKind::ItemShown => {
                records[at] = ItemRecord {
                    item,
                    rated: None,
                    marked: None,
                    read_ms: None,
                };
            }
            EventKind::ItemMarked(d) => records[at].marked = Some(*d),
            EventKind::ItemFlipped | EventKind::AnswerSubmitted(_) => {
                revealed = Some((item, e.t_ms, paused_total));
            }
            EventKind::ItemRated(r) => {
                records[at].rated = Some(*r);
                records[at].read_ms = revealed
                    .filter(|(i, _, _)| *i == item)
                    .map(|(_, t, p)| e.t_ms.saturating_sub(t).saturating_sub(paused_total - p));
            }
            _ => {}
        }
    }
    records
}

/// Why a rating is suggested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// A known item was read quickly and recalled: it looks easy.
    KnownAndFast,
    /// A new item was read quickly and still rated well: it was probably not read.
    NewAndFast,
    /// Reading took much longer than the words should need: a sign of struggle.
    Slow,
}

/// A suggested rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suggestion {
    /// The rating to consider instead of the one given.
    pub rating: Rating,
    /// Why.
    pub reason: Reason,
}

/// The suggestion for one item, if its reading time says something its rating does not.
///
/// A fast read of a known item rated Good becomes Easy; a fast read of a new item rated Good or Easy
/// becomes Hard (it went by too fast to have been read). A slow read lowers a Good to Hard and an
/// Easy to Good: the struggle shows even if the answer came in the end. A low rating is never softened.
pub fn suggest(
    words: usize,
    read_ms: u32,
    previous: Option<Rating>,
    rated: Rating,
    p: &ReadingParams,
) -> Option<Suggestion> {
    if words < p.min_words {
        return None;
    }
    // read_ms against expected_ms * percent / 100, in whole numbers.
    let (actual, expected) = (u64::from(read_ms) * 100, u64::from(expected_ms(words, p)));
    let by = |rating, reason| Some(Suggestion { rating, reason });
    if actual < expected * u64::from(p.fast_percent) {
        return match (previous, rated) {
            (Some(_), Rating::Good) => by(Rating::Easy, Reason::KnownAndFast),
            (None, Rating::Good | Rating::Easy) => by(Rating::Hard, Reason::NewAndFast),
            _ => None,
        };
    }
    if actual > expected * u64::from(p.slow_percent) {
        return match rated {
            Rating::Easy => by(Rating::Good, Reason::Slow),
            Rating::Good => by(Rating::Hard, Reason::Slow),
            _ => None,
        };
    }
    None
}
