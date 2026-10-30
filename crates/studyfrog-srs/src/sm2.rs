//! SM-2 (Wozniak, 1990) as pure functions: the baseline and the simple user mode.
//!
//! Source: <https://super-memory.com/english/ol/sm2.htm>. The original grades recall 0 to 5; we map
//! our four ratings to Again = 1, Hard = 3, Good = 4, Easy = 5. A grade below 3 restarts the
//! repetitions and leaves the ease factor unchanged. The "repeat until at least 4 within the
//! session" step is session logic and belongs to the rehearsal engine.

use studyfrog_core::memory::Sm2State;
use studyfrog_core::review::Rating;

/// Starting ease factor.
pub const INITIAL_EASE: f32 = 2.5;
/// Lowest allowed ease factor.
pub const MIN_EASE: f32 = 1.3;

/// State of an item that has never been reviewed.
pub fn sm2_initial() -> Sm2State {
    Sm2State {
        ease: INITIAL_EASE,
        repetitions: 0,
        interval_days: 0,
    }
}

/// The 0 to 5 grade for a rating.
pub fn sm2_grade(rating: Rating) -> u8 {
    match rating {
        Rating::Again => 1,
        Rating::Hard => 3,
        Rating::Good => 4,
        Rating::Easy => 5,
    }
}

fn next_ease(ease: f32, grade: u8) -> f32 {
    let miss = f32::from(5 - grade);
    (ease + 0.1 - miss * (0.08 + miss * 0.02)).max(MIN_EASE)
}

fn next_interval(repetitions: u32, previous: u32, ease: f32) -> u32 {
    match repetitions {
        1 => 1,
        2 => 6,
        _ => (previous as f32 * ease).ceil() as u32,
    }
}

/// State after reviewing with `rating`.
pub fn sm2_next(prev: &Sm2State, rating: Rating, maximum_interval_days: u32) -> Sm2State {
    let grade = sm2_grade(rating);
    if grade < 3 {
        return Sm2State {
            ease: prev.ease,
            repetitions: 0,
            interval_days: 1,
        };
    }
    let ease = next_ease(prev.ease, grade);
    let repetitions = prev.repetitions + 1;
    let days = next_interval(repetitions, prev.interval_days, ease).clamp(1, maximum_interval_days);
    Sm2State {
        ease,
        repetitions,
        interval_days: days,
    }
}
