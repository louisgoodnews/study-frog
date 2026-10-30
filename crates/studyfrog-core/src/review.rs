//! Review history: the input to scheduling.

use crate::time::UnixMs;
use crate::uid::Uid;

/// Self-assessed recall quality. Ordered from worst to best.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Rating {
    /// Forgotten.
    Again = 1,
    /// Recalled with serious effort.
    Hard = 2,
    /// Recalled correctly.
    Good = 3,
    /// Recalled effortlessly.
    Easy = 4,
}

/// One recorded recall attempt of an item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Review {
    /// The reviewed item (note, card or question).
    pub item: Uid,
    /// Rating given.
    pub rating: Rating,
    /// When the review happened.
    pub at: UnixMs,
    /// Days since the previous review of this item; `0.0` for the first review.
    pub elapsed_days: f32,
}

/// Numeric value stored in the database and used by SRS formulas (1 to 4).
pub fn rating_value(rating: Rating) -> u8 {
    rating as u8
}

/// Inverse of [`rating_value`]; `None` for anything outside 1 to 4.
pub fn rating_from_value(value: u8) -> Option<Rating> {
    match value {
        1 => Some(Rating::Again),
        2 => Some(Rating::Hard),
        3 => Some(Rating::Good),
        4 => Some(Rating::Easy),
        _ => None,
    }
}
