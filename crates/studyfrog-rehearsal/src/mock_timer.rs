//! The strict exam clock (STORY-0185): warnings at 10 and 2 minutes left, and automatic submission
//! when time runs out. Pure: the caller supplies the elapsed seconds.

/// The clock's state at a moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerStatus {
    /// Plenty of time left; seconds remaining.
    Running {
        /// Seconds left.
        remaining_sec: u32,
    },
    /// Ten minutes or fewer left, more than two.
    TenMinutesLeft,
    /// Two minutes or fewer left.
    TwoMinutesLeft,
    /// No time left: the exam is submitted automatically.
    Expired,
}

/// A warning the caller should show once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAlert {
    /// Ten minutes left.
    TenMinutes,
    /// Two minutes left.
    TwoMinutes,
    /// Time is up: submit now.
    Expired,
}

/// The clock's state for a limit and elapsed time, both in seconds.
pub fn timer_status(limit_sec: u32, elapsed_sec: u32) -> TimerStatus {
    let remaining = limit_sec.saturating_sub(elapsed_sec);
    if remaining == 0 {
        TimerStatus::Expired
    } else if remaining <= 120 {
        TimerStatus::TwoMinutesLeft
    } else if remaining <= 600 {
        TimerStatus::TenMinutesLeft
    } else {
        TimerStatus::Running {
            remaining_sec: remaining,
        }
    }
}

/// Alerts crossed between two elapsed times (`before` exclusive, `after` inclusive). Each threshold
/// is reported once, in order, so a long gap between two checks does not skip the expiry.
pub fn alerts_between(limit_sec: u32, before: u32, after: u32) -> Vec<TimerAlert> {
    let thresholds = [
        (limit_sec.saturating_sub(600), TimerAlert::TenMinutes),
        (limit_sec.saturating_sub(120), TimerAlert::TwoMinutes),
        (limit_sec, TimerAlert::Expired),
    ];
    thresholds
        .into_iter()
        .filter(|&(at, _)| before < at && at <= after)
        .map(|(_, alert)| alert)
        .collect()
}
