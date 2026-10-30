//! Memory model state of one item, as computed by the scheduler.

/// FSRS memory state. Persisted by the store, advanced by `studyfrog-srs`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemState {
    /// Days for retrievability to fall from 100% to 90%.
    pub stability: f32,
    /// Item difficulty in `[1, 10]`.
    pub difficulty: f32,
}
