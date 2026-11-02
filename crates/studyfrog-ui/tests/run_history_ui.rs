#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Runs tab of the reports: mode, pacing, time and answers a minute, filtered by mode; and the line in the review.

use studyfrog_core::pacing::Pacing;
use studyfrog_ui::run_time::{pacing_text, run_line};

#[test]
fn run_lines_say_the_pacing_the_time_used_and_the_answers_a_minute() {
    assert_eq!(pacing_text(Pacing::Free), "No clock");
    assert_eq!(pacing_text(Pacing::Countdown(600)), "Countdown of 10:00");
    assert_eq!(
        run_line(Pacing::Countdown(600), 582_000, 12),
        "Countdown of 10:00, used 09:42, 1.2 answers a minute"
    );
    assert_eq!(run_line(Pacing::CountUp, 0, 0), "Count up, used 00:00");
}
