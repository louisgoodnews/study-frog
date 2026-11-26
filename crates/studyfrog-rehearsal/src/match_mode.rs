//! Match Mode state machine, tile pairing mechanics, and monotonic timer.
//!
//! Complies with Invariant P1: zero inherent impl blocks; domain logic is
//! implemented entirely in pure free functions.

/// The role/kind of a match tile: either prompt (question/term) or answer (definition).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    /// Front of the card (prompt/term).
    Prompt,
    /// Back of the card (answer/definition).
    Answer,
}

/// The visual and interactive status of a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum TileStatus {
    Idle,
    Selected,
    Matched,
    Mismatch { until_ms: u32 },
}

/// A single interactive tile within the match grid.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct MatchTile {
    pub id: usize,
    pub pair_id: usize,
    pub text: String,
    pub kind: TileKind,
    pub status: TileStatus,
}

/// Full state of an active match mode session drill.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct MatchModeSession {
    pub tiles: Vec<MatchTile>,
    pub selected_first: Option<usize>,
    pub pairs_remaining: usize,
    pub start_t_ms: u32,
    pub elapsed_ms: u32,
    pub penalty_ms: u32,
    pub streak: u32,
    pub best_streak: u32,
    pub mismatches: u32,
    pub completed: bool,
}

/// Result of a user tile selection attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum MatchOutcome {
    Selected(usize),
    Deselected,
    Matched {
        pair_id: usize,
        streak: u32,
    },
    Mismatch {
        penalty_ms: u32,
    },
    Completed {
        total_time_ms: u32,
        best_streak: u32,
    },
    Ignored,
}

fn next_xorshift64(state: &mut u64) -> u64 {
    let mut x = if *state == 0 {
        0x853c_49e6_748f_ea9b
    } else {
        *state
    };
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn push_tile(v: &mut Vec<MatchTile>, pair_id: usize, text: String, kind: TileKind) {
    v.push(MatchTile {
        id: 0,
        pair_id,
        text,
        kind,
        status: TileStatus::Idle,
    });
}

/// Creates a new match mode session from pairs of `(prompt, answer)`.
pub fn match_session_new(
    pairs: &[(String, String)],
    start_t_ms: u32,
    seed: u64,
) -> MatchModeSession {
    let take_count = pairs.len().min(8);
    let mut tiles = Vec::with_capacity(take_count * 2);
    for (pair_id, (prompt, answer)) in pairs.iter().take(take_count).enumerate() {
        push_tile(&mut tiles, pair_id, prompt.clone(), TileKind::Prompt);
        push_tile(&mut tiles, pair_id, answer.clone(), TileKind::Answer);
    }
    let mut rng = seed;
    let n = tiles.len();
    if n > 1 {
        for i in (1..n).rev() {
            let r = (next_xorshift64(&mut rng) as usize) % (i + 1);
            tiles.swap(i, r);
        }
    }
    for (idx, tile) in tiles.iter_mut().enumerate() {
        tile.id = idx;
    }
    MatchModeSession {
        tiles,
        selected_first: None,
        pairs_remaining: take_count,
        start_t_ms,
        elapsed_ms: 0,
        penalty_ms: 0,
        streak: 0,
        best_streak: 0,
        mismatches: 0,
        completed: take_count == 0,
    }
}

/// Updates the monotonic timer and clears any expired mismatch feedback states.
pub fn update_match_timer(session: &mut MatchModeSession, current_t_ms: u32) {
    if !session.completed {
        let raw = current_t_ms.saturating_sub(session.start_t_ms);
        session.elapsed_ms = raw.saturating_add(session.penalty_ms);
    }
    for tile in &mut session.tiles {
        if matches!(tile.status, TileStatus::Mismatch { until_ms } if current_t_ms >= until_ms) {
            tile.status = TileStatus::Idle;
        }
    }
}

fn handle_pair(s: &mut MatchModeSession, a: usize, b: usize, now: u32) -> MatchOutcome {
    if s.tiles[a].pair_id == s.tiles[b].pair_id {
        s.tiles[a].status = TileStatus::Matched;
        s.tiles[b].status = TileStatus::Matched;
        s.streak = s.streak.saturating_add(1);
        s.best_streak = s.best_streak.max(s.streak);
        s.pairs_remaining = s.pairs_remaining.saturating_sub(1);
        if s.pairs_remaining == 0 {
            s.completed = true;
            s.elapsed_ms = now
                .saturating_sub(s.start_t_ms)
                .saturating_add(s.penalty_ms);
            MatchOutcome::Completed {
                total_time_ms: s.elapsed_ms,
                best_streak: s.best_streak,
            }
        } else {
            MatchOutcome::Matched {
                pair_id: s.tiles[b].pair_id,
                streak: s.streak,
            }
        }
    } else {
        s.streak = 0;
        s.mismatches = s.mismatches.saturating_add(1);
        s.penalty_ms = s.penalty_ms.saturating_add(1000);
        s.elapsed_ms = s.elapsed_ms.saturating_add(1000);
        let until = now.saturating_add(1000);
        s.tiles[a].status = TileStatus::Mismatch { until_ms: until };
        s.tiles[b].status = TileStatus::Mismatch { until_ms: until };
        MatchOutcome::Mismatch { penalty_ms: 1000 }
    }
}

/// Handles user tile selection at the given timestamp.
pub fn select_tile(
    session: &mut MatchModeSession,
    tile_idx: usize,
    current_t_ms: u32,
) -> MatchOutcome {
    if session.completed || tile_idx >= session.tiles.len() {
        return MatchOutcome::Ignored;
    }
    update_match_timer(session, current_t_ms);
    if session.tiles[tile_idx].status == TileStatus::Matched {
        return MatchOutcome::Ignored;
    }
    for tile in &mut session.tiles {
        if matches!(tile.status, TileStatus::Mismatch { .. }) {
            tile.status = TileStatus::Idle;
        }
    }
    match session.selected_first {
        None => {
            session.tiles[tile_idx].status = TileStatus::Selected;
            session.selected_first = Some(tile_idx);
            MatchOutcome::Selected(tile_idx)
        }
        Some(first) if first == tile_idx => {
            session.tiles[tile_idx].status = TileStatus::Idle;
            session.selected_first = None;
            MatchOutcome::Deselected
        }
        Some(first) => {
            session.selected_first = None;
            handle_pair(session, first, tile_idx, current_t_ms)
        }
    }
}

/// Computes the match score based on matched pairs, streak, speed, and penalties.
pub fn compute_match_score(session: &MatchModeSession) -> u32 {
    let total_pairs = session.tiles.len() / 2;
    if total_pairs == 0 {
        return 0;
    }
    let matched = (total_pairs.saturating_sub(session.pairs_remaining) as u32) * 1000;
    let streak = session.best_streak * 250;
    let penalty = session.mismatches * 200;
    let speed = if session.completed {
        60_000u32.saturating_sub(session.elapsed_ms) / 12
    } else {
        0
    };
    let min = if session.completed { 100 } else { 0 };
    matched
        .saturating_add(streak)
        .saturating_add(speed)
        .saturating_sub(penalty)
        .max(min)
}

/// Formats elapsed time in milliseconds into `MM:SS.d` display format.
pub fn format_match_time(elapsed_ms: u32) -> String {
    let (m, s, t) = (
        elapsed_ms / 60_000,
        (elapsed_ms % 60_000) / 1000,
        (elapsed_ms % 1000) / 100,
    );
    format!("{m:02}:{s:02}.{t}")
}
