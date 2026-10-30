//! The "today" a daily budget is counted in. The shell decides where a day starts (local midnight,
//! or a rollover hour); nothing here reads a clock or a time zone.

use studyfrog_core::time::UnixMs;

/// A half-open day `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayWindow {
    /// First millisecond of the day.
    pub start: UnixMs,
    /// First millisecond of the next day.
    pub end: UnixMs,
}

/// The 24-hour window beginning at `start`.
pub fn day_from(start: UnixMs) -> DayWindow {
    DayWindow {
        start,
        end: UnixMs(start.0 + 86_400_000),
    }
}
