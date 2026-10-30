//! Deterministic interval fuzzing (Anki-style) so cards created together do not clump.
//! The caller derives `unit` in `[0, 1)` from a seed; nothing here is random.

/// `(start, end, factor)` bands of the fuzz window.
const BANDS: [(f32, f32, f32); 3] = [
    (2.5, 7.0, 0.15),
    (7.0, 20.0, 0.1),
    (20.0, f32::INFINITY, 0.05),
];

/// Intervals below this are never fuzzed.
const MIN_FUZZED: f32 = 2.5;

fn delta(interval: f32) -> f32 {
    BANDS.iter().fold(1.0, |acc, (start, end, f)| {
        acc + f * (interval.min(*end) - start).max(0.0)
    })
}

/// Inclusive whole-day window around `interval`, clamped to `[2, maximum]`.
pub fn fuzz_bounds(interval: f32, maximum: u32) -> (u32, u32) {
    let d = delta(interval);
    let high = ((interval + d).round() as u32).min(maximum);
    let low = ((interval - d).round().max(2.0) as u32).min(high);
    (low, high)
}

/// Pick a day inside the fuzz window using `unit` in `[0, 1)`.
pub fn fuzz_interval(interval: f32, unit: f32, maximum: u32) -> u32 {
    if interval < MIN_FUZZED {
        return (interval.round() as u32).clamp(1, maximum);
    }
    let (low, high) = fuzz_bounds(interval, maximum);
    let span = (high - low + 1) as f32;
    (low + (unit.clamp(0.0, 0.999_999) * span) as u32).min(high)
}
