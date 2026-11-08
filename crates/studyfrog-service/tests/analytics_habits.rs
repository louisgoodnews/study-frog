#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The hour and weekday profile and within-session fatigue from a fixture vault (STORY-0102, STORY-0103).

mod analytics_common;

use analytics_common::*;
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::time::UnixMs;
use studyfrog_service::reports::{build_reports, build_reports_with};

const HOUR: i64 = 3_600_000;

/// Two cards rated on Monday at 09:00 UTC in one run: shown at 0 s and 10 s, revealed after 2 s and 4 s.
fn morning(label: &str) -> (Dir, studyfrog_service::session::VaultSession) {
    let (d, s) = vault(
        label,
        &[],
        &[(10, "", "A", "q\n???\na"), (11, "", "B", "q\n???\na")],
    );
    let start = T0 + 9 * HOUR;
    rate_all(
        &s,
        &[(10, start + 3_000, Good), (11, start + 15_000, Again)],
    );
    let mut events = shown_flipped_rated(0, 10, 2_000, "3");
    events.extend(shown_flipped_rated(10_000, 11, 4_000, "1"));
    run_events(&s, 900, start, &events);
    (d, s)
}

#[test]
fn reviews_and_answer_times_are_placed_in_the_local_hour_and_weekday() {
    let (_d, s) = morning("habits-clock");
    let utc = build_reports(&s, UnixMs(T0 + 2 * 86_400_000), 0).unwrap();
    let c = &utc.analytics.chronotype;
    assert_eq!(
        (c.hours[9].reviews, c.hours[9].recalled, c.hours[9].answers),
        (2, 1, 2)
    );
    assert_eq!(c.hours[9].median_ms, Some(2_000));
    assert_eq!(c.weekdays[0].reviews, 2, "Monday");
    assert_eq!(c.peak, None, "far below the minimum of reviews per hour");
    // Eleven hours east it is 20:00 on Monday; fifteen hours west it is 18:00 on Sunday.
    let east = build_reports(&s, UnixMs(T0 + 2 * 86_400_000), 660).unwrap();
    assert_eq!(east.analytics.chronotype.hours[20].reviews, 2);
    let west = build_reports_with(&s, UnixMs(T0 + 2 * 86_400_000), &|_| -900).unwrap();
    assert_eq!(west.analytics.chronotype.hours[18].answers, 2);
    assert_eq!(west.analytics.chronotype.weekdays[6].reviews, 2, "Sunday");
}

#[test]
fn the_run_log_feeds_the_fatigue_report() {
    let (_d, s) = morning("habits-run");
    let r = build_reports(&s, UnixMs(T0 + 2 * 86_400_000), 0).unwrap();
    let f = &r.analytics.fatigue;
    assert_eq!(f.sessions, 1);
    assert_eq!((f.by_time[0].reviews, f.by_time[0].recalled), (2, 1));
    assert_eq!((f.by_position[0].reviews, f.baseline.reviews), (2, 2));
    assert_eq!(
        f.by_time[0].median_ms,
        Some(2_000),
        "nearest rank of 2 s and 4 s"
    );
    assert_eq!(f.inflection, None);
}
