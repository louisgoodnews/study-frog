//! The types and constants of familiarity (`docs/specs/familiarity.md`).

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;

/// The constants of the formula (section 7 of the specification).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FamParams {
    /// Share of the remaining gap to 1.0 that an Easy rating closes (before speed and mark).
    pub gain_easy: f32,
    /// The same for Good.
    pub gain_good: f32,
    /// Share of the value a Hard rating takes away.
    pub loss_hard: f32,
    /// Share of the value an Again rating takes away.
    pub loss_again: f32,
    /// Share of the gap a counted view closes (before speed).
    pub view_gain: f32,
    /// Days without contact in which the value halves.
    pub half_life_days: f32,
    /// If above 0, the half-life is at least this many times the longest gap between two ratings so far, so an item
    /// the user can leave alone for months does not fall as fast as one reviewed daily. 0 is the specification.
    pub gap_factor: f32,
    /// Lowest and highest speed factor of a rating.
    pub speed_bounds: (f32, f32),
    /// Lowest speed factor of a view (the highest is the rating's).
    pub view_speed_min: f32,
    /// Highest value of a note.
    pub note_cap: f32,
    /// Gain factors for an `easy` and a `difficult` mark.
    pub mark_factors: (f32, f32),
    /// Ratings that give full confidence, and views that give a note half of it.
    pub full_confidence: (f32, f32),
    /// Views shorter than this many milliseconds are ignored.
    pub min_view_ms: u32,
    /// At most one view counts in this many milliseconds.
    pub view_gap_ms: i64,
    /// Days before an exam or due date over which gains are damped to the floor, and that floor (a share of a full gain).
    pub exam_ramp: (f32, f32),
}

/// The defaults of the specification.
pub fn fam_params_default() -> FamParams {
    FamParams {
        gain_easy: 0.45,
        gain_good: 0.30,
        loss_hard: 0.25,
        loss_again: 0.50,
        view_gain: 0.02,
        half_life_days: 365.0,
        gap_factor: 0.0,
        speed_bounds: (0.5, 1.5),
        view_speed_min: 0.25,
        note_cap: 0.8,
        mark_factors: (1.1, 0.8),
        full_confidence: (5.0, 10.0),
        min_view_ms: 1000,
        view_gap_ms: 3_600_000,
        exam_ramp: (14.0, 0.5),
    }
}

/// Something that happened to an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamEvent {
    /// A rating, with the time the answer side was on screen and the time it should take at the reading speed.
    Rating {
        /// When.
        at: UnixMs,
        /// The rating.
        rating: Rating,
        /// Time on the answer side, if measured.
        answer_ms: Option<u32>,
        /// Expected reading time, if known.
        expected_ms: Option<u32>,
    },
    /// A visit to the item page.
    View {
        /// When it ended.
        at: UnixMs,
        /// Active time.
        active_ms: u32,
        /// Expected reading time of the item's text, if known.
        expected_ms: Option<u32>,
    },
    /// An edit: contact, but no change of the value.
    Edit {
        /// When.
        at: UnixMs,
    },
}

/// What is known about the item besides its history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FamContext {
    /// A note has no ratings: its value is capped and its confidence lower.
    pub is_note: bool,
    /// The item's difficulty mark.
    pub mark: Option<Difficulty>,
    /// The item's status is Complete: the fall without contact stops.
    pub done: bool,
    /// The exam date of the item's stack: the fall stops there.
    pub exam: Option<UnixMs>,
}

/// A value and how much history stands behind it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Familiarity {
    /// 0.0 to 1.0.
    pub value: f32,
    /// 0.0 to 1.0; below 0.4 the screen says "few reviews".
    pub confidence: f32,
}
