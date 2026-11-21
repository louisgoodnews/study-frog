//! Memory Mode timing and results (STORY-0175): stopwatch or time attack, the danger level as the
//! countdown runs out, and the victory summary with speed, bonus and personal best. Pure.

use crate::memory::{MemoryOutcome, MemorySession};
use crate::memory_deck::FlipState;

/// Countdown choices for time attack, in seconds.
pub const TIME_ATTACK_OPTIONS_SEC: [u32; 3] = [60, 90, 120];
/// Bonus points per second left on a time-attack clear.
pub const BONUS_PER_SECOND: u32 = 10;

/// How a Memory Mode game is timed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryTimer {
    /// Open-ended: the fastest clear wins.
    #[default]
    Stopwatch,
    /// A fixed countdown, in seconds.
    TimeAttack(u32),
}

/// How urgent the countdown should look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerLevel {
    /// No countdown, or plenty of time left.
    Calm,
    /// Under 30 seconds left.
    Warning,
    /// Under 10 seconds left.
    Critical,
}

/// What the student sees after a clear.
#[derive(Debug, Clone, PartialEq)]
pub struct VictorySummary {
    /// Time taken, in milliseconds.
    pub elapsed_ms: u64,
    /// Turns taken (two flips each).
    pub moves: u32,
    /// Pairs found.
    pub pairs: u32,
    /// Pairs found per turn, as a percentage.
    pub accuracy_pct: f32,
    /// Pairs per minute.
    pub speed_score: f32,
    /// Bonus for time left on a time-attack clear; zero on a stopwatch.
    pub bonus_points: u32,
    /// The longest run of matches.
    pub best_streak: u32,
    /// Whether this beats the previous best time (or there was none).
    pub personal_best: bool,
}

/// Danger level for a countdown at `elapsed_ms`.
pub fn danger_level(timer: MemoryTimer, elapsed_ms: u64) -> DangerLevel {
    let MemoryTimer::TimeAttack(limit_sec) = timer else {
        return DangerLevel::Calm;
    };
    let remaining = (u64::from(limit_sec) * 1000).saturating_sub(elapsed_ms);
    if remaining <= 10_000 {
        DangerLevel::Critical
    } else if remaining <= 30_000 {
        DangerLevel::Warning
    } else {
        DangerLevel::Calm
    }
}

/// The victory summary, once the session is completed. `None` while playing or after a timeout.
pub fn victory_summary(
    s: &MemorySession,
    timer: MemoryTimer,
    previous_best_ms: Option<u64>,
) -> Option<VictorySummary> {
    let Some(MemoryOutcome::Completed { elapsed_ms }) = s.outcome else {
        return None;
    };
    let pairs = u32::try_from(
        s.tiles
            .iter()
            .filter(|t| t.state == FlipState::Matched)
            .count()
            / 2,
    )
    .unwrap_or(u32::MAX);
    let accuracy_pct = if s.moves == 0 {
        100.0
    } else {
        pairs as f32 * 100.0 / s.moves as f32
    };
    let minutes = elapsed_ms as f32 / 60_000.0;
    let speed_score = if minutes > 0.0 {
        pairs as f32 / minutes
    } else {
        0.0
    };
    let bonus_points = match timer {
        MemoryTimer::TimeAttack(limit_sec) => {
            let left_ms = (u64::from(limit_sec) * 1000).saturating_sub(elapsed_ms);
            u32::try_from(left_ms / 1000).unwrap_or(u32::MAX) * BONUS_PER_SECOND
        }
        MemoryTimer::Stopwatch => 0,
    };
    Some(VictorySummary {
        elapsed_ms,
        moves: s.moves,
        pairs,
        accuracy_pct,
        speed_score,
        bonus_points,
        best_streak: s.best_streak,
        personal_best: previous_best_ms.is_none_or(|best| elapsed_ms < best),
    })
}

/// The encouraging line shown after a timeout, before the cards are revealed.
pub fn defeat_message() -> &'static str {
    "Time is up. Here are the cards: try again and beat your time."
}
