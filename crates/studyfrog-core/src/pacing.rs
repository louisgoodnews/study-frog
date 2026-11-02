//! How the clock figures in a run (`docs/specs/rehearsal-run.md`, "Pacing").

/// The pacing of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Pacing {
    /// No limit and no clock shown: the run as it always was.
    #[default]
    Free,
    /// The run ends by itself after this many seconds of active time.
    Countdown(u32),
    /// No limit; the elapsed time is shown.
    CountUp,
}

/// Shortest countdown, in seconds.
pub const MIN_COUNTDOWN_SECS: u32 = 60;
/// Longest countdown, in seconds (three hours).
pub const MAX_COUNTDOWN_SECS: u32 = 10_800;

/// The word stored for a pacing.
pub fn pacing_name(p: Pacing) -> &'static str {
    match p {
        Pacing::Free => "free",
        Pacing::Countdown(_) => "countdown",
        Pacing::CountUp => "count_up",
    }
}

/// The seconds of a countdown; `None` for the other pacings.
pub fn countdown_secs(p: Pacing) -> Option<u32> {
    match p {
        Pacing::Countdown(secs) => Some(secs),
        _ => None,
    }
}

/// Rebuild a pacing from its stored word and seconds; `None` if they do not fit (an unknown word, a countdown
/// without seconds or outside the limits).
pub fn pacing_from_parts(name: &str, secs: Option<u32>) -> Option<Pacing> {
    match (name, secs) {
        ("free", _) => Some(Pacing::Free),
        ("count_up", _) => Some(Pacing::CountUp),
        ("countdown", Some(s)) if (MIN_COUNTDOWN_SECS..=MAX_COUNTDOWN_SECS).contains(&s) => {
            Some(Pacing::Countdown(s))
        }
        _ => None,
    }
}

/// The active time a countdown allows, in milliseconds.
pub fn limit_ms(p: Pacing) -> Option<u32> {
    countdown_secs(p).map(|s| s.saturating_mul(1000))
}
