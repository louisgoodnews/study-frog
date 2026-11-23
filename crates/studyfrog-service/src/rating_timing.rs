//! The timing of one rating: what familiarity and telemetry record about it. Plain data.

/// The timing of a rating: for familiarity, the answer's time on screen and the reading time it
/// should need; for telemetry, the latency, the dwell, the item's size and its semantic score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RatingTiming {
    /// Milliseconds the answer was on screen before the rating, paused time excluded (familiarity).
    pub answer_ms: u32,
    /// Milliseconds the answer's words should take to read at the user's speed.
    pub expected_ms: u32,
    /// Milliseconds from the item being shown to the first response (the flip, or the submitted
    /// answer), paused time excluded.
    pub latency_ms: u32,
    /// Milliseconds from the item being shown to its rating, paused time excluded.
    pub dwell_ms: u32,
    /// Words in the item's question and reference answer.
    pub text_words: u32,
    /// The semantic similarity of the typed answer, in `[0, 1]`, when one was judged.
    pub semantic: Option<f32>,
}
