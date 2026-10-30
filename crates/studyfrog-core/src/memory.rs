//! Memory model state of one item, as computed by the scheduler.

/// FSRS memory state. Persisted by the store, advanced by `studyfrog-srs`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemState {
    /// Days for retrievability to fall from 100% to 90%.
    pub stability: f32,
    /// Item difficulty in `[1, 10]`.
    pub difficulty: f32,
}

/// SM-2 state of one item (the simple scheduling mode).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sm2State {
    /// Ease factor, never below 1.3.
    pub ease: f32,
    /// Consecutive successful repetitions.
    pub repetitions: u32,
    /// Current interval in days; 0 before the first successful review.
    pub interval_days: u32,
}
