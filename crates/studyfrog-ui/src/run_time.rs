//! How a run is described in words: its pacing, the time it used and the answers a minute. Pure.

use studyfrog_core::pacing::Pacing;
use studyfrog_service::run_history::answers_per_minute;

use crate::run_clock::mmss;

/// "Countdown of 10:00", "Count up" or "No clock".
pub fn pacing_text(p: Pacing) -> String {
    match p {
        Pacing::Free => "No clock".to_owned(),
        Pacing::CountUp => "Count up".to_owned(),
        Pacing::Countdown(s) => format!("Countdown of {}", mmss(s * 1000)),
    }
}

/// "Countdown of 10:00, used 09:42, 12.4 answers a minute".
pub fn run_line(p: Pacing, used_ms: u32, rated: u32) -> String {
    let rate = answers_per_minute(rated, used_ms)
        .map_or(String::new(), |r| format!(", {r:.1} answers a minute"));
    format!("{}, used {}{rate}", pacing_text(p), mmss(used_ms))
}
