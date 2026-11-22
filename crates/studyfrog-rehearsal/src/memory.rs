//! Memory Mode state machine (STORY-0173): two-turn flips, match or mismatch, a mismatch display
//! delay, move and streak counters, and an optional time limit. Pure: the caller supplies the clock
//! as elapsed milliseconds and feeds in flips.

use crate::memory_deck::{FlipState, MemoryTile, build_tiles};

/// One Memory Mode session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySession {
    /// The grid, in display order.
    pub tiles: Vec<MemoryTile>,
    /// The first tile of a turn, waiting for its partner.
    pub open: Option<usize>,
    /// The two tiles currently shown as a mismatch.
    pub mismatch: Option<(usize, usize)>,
    /// Elapsed milliseconds at which the mismatch hides again.
    pub mismatch_until_ms: u64,
    /// How long a mismatch stays visible, in milliseconds.
    pub mismatch_delay_ms: u64,
    /// Completed turns (two flips each).
    pub moves: u32,
    /// Matches in a row.
    pub streak: u32,
    /// The longest streak so far.
    pub best_streak: u32,
    /// Elapsed milliseconds.
    pub elapsed_ms: u64,
    /// Time limit for time-attack, or `None` for a plain stopwatch.
    pub limit_ms: Option<u64>,
    /// How the session ended, once it has.
    pub outcome: Option<MemoryOutcome>,
}

/// How a session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOutcome {
    /// Every pair was found, in this many milliseconds.
    Completed {
        /// Time taken.
        elapsed_ms: u64,
    },
    /// The time limit ran out first.
    TimedOut,
}

/// Why a flip was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlipError {
    /// The session has ended.
    Finished,
    /// A mismatch is still showing; wait for it to hide.
    Blocked,
    /// No such tile.
    NoSuchTile,
    /// The tile is already face up or matched.
    NotFaceDown,
}

/// A new session for the first `pairs` cards, shuffled by `seed`.
pub fn new_memory_session(
    cards: &[(String, String)],
    pairs: usize,
    seed: u64,
    mismatch_delay_ms: u64,
    limit_ms: Option<u64>,
) -> MemorySession {
    MemorySession {
        tiles: build_tiles(cards, pairs, seed),
        open: None,
        mismatch: None,
        mismatch_until_ms: 0,
        mismatch_delay_ms,
        moves: 0,
        streak: 0,
        best_streak: 0,
        elapsed_ms: 0,
        limit_ms,
        outcome: None,
    }
}

fn is_complete(s: &MemorySession) -> bool {
    !s.tiles.is_empty() && s.tiles.iter().all(|t| t.state == FlipState::Matched)
}

fn with_state(s: &MemorySession, idx: usize, state: FlipState) -> Vec<MemoryTile> {
    let mut tiles = s.tiles.clone();
    tiles[idx].state = state;
    tiles
}

/// Turn a face-up tile over. The first flip of a turn opens it; the second resolves the turn: a
/// matching pair is removed, a mismatch stays visible for the display delay.
pub fn flip_tile(s: &MemorySession, idx: usize) -> Result<MemorySession, FlipError> {
    if s.outcome.is_some() {
        return Err(FlipError::Finished);
    }
    if s.mismatch.is_some() {
        return Err(FlipError::Blocked);
    }
    let tile = s.tiles.get(idx).ok_or(FlipError::NoSuchTile)?;
    if tile.state != FlipState::FaceDown {
        return Err(FlipError::NotFaceDown);
    }
    let mut next = s.clone();
    let Some(first) = s.open else {
        next.tiles = with_state(s, idx, FlipState::FaceUp);
        next.open = Some(idx);
        return Ok(next);
    };
    next.moves += 1;
    next.open = None;
    if s.tiles[first].pair == tile.pair {
        next.tiles = with_state(s, idx, FlipState::Matched);
        next.tiles[first].state = FlipState::Matched;
        next.streak += 1;
        next.best_streak = next.best_streak.max(next.streak);
        if is_complete(&next) {
            next.outcome = Some(MemoryOutcome::Completed {
                elapsed_ms: s.elapsed_ms,
            });
        }
    } else {
        next.tiles = with_state(s, idx, FlipState::MismatchWait);
        next.tiles[first].state = FlipState::MismatchWait;
        next.streak = 0;
        next.mismatch = Some((first, idx));
        next.mismatch_until_ms = s.elapsed_ms.saturating_add(s.mismatch_delay_ms);
    }
    Ok(next)
}

/// Advance the clock to `now_ms`: hide a mismatch whose delay has passed, and end the session when
/// the time limit is reached. Time never runs backwards, and a finished session's clock stops: its
/// time is the time it ended at. A timeout turns the unmatched cards face up.
pub fn tick_memory_timer(s: &MemorySession, now_ms: u64) -> MemorySession {
    if s.outcome.is_some() {
        return s.clone();
    }
    let mut next = s.clone();
    next.elapsed_ms = now_ms.max(s.elapsed_ms);
    if let Some((a, b)) = s.mismatch
        && next.elapsed_ms >= s.mismatch_until_ms
    {
        next.tiles[a].state = FlipState::FaceDown;
        next.tiles[b].state = FlipState::FaceDown;
        next.mismatch = None;
    }
    if let Some(limit) = s.limit_ms
        && next.elapsed_ms >= limit
    {
        return time_out(next);
    }
    next
}

/// End the session as timed out: every unmatched card is turned face up so the pairs can be seen.
fn time_out(mut s: MemorySession) -> MemorySession {
    for tile in s.tiles.iter_mut().filter(|t| t.state != FlipState::Matched) {
        tile.state = FlipState::FaceUp;
    }
    s.mismatch = None;
    s.open = None;
    s.outcome = Some(MemoryOutcome::TimedOut);
    s
}
