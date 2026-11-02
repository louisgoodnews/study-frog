#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The pacing of a run: stored words, limits and the time a countdown allows.

use studyfrog_core::pacing::{
    MAX_COUNTDOWN_SECS, MIN_COUNTDOWN_SECS, Pacing, countdown_secs, limit_ms, pacing_from_parts,
    pacing_name,
};

#[test]
fn every_pacing_has_a_stored_word_and_reads_back() {
    for p in [Pacing::Free, Pacing::CountUp, Pacing::Countdown(900)] {
        assert_eq!(
            pacing_from_parts(pacing_name(p), countdown_secs(p)),
            Some(p),
            "{p:?}"
        );
    }
    assert_eq!(Pacing::default(), Pacing::Free);
    assert_eq!(
        [Pacing::Free, Pacing::CountUp, Pacing::Countdown(60)].map(pacing_name),
        ["free", "count_up", "countdown"]
    );
}

#[test]
fn a_countdown_needs_seconds_inside_the_limits_and_other_words_are_refused() {
    assert_eq!(pacing_from_parts("countdown", None), None);
    assert_eq!(
        pacing_from_parts("countdown", Some(MIN_COUNTDOWN_SECS - 1)),
        None
    );
    assert_eq!(
        pacing_from_parts("countdown", Some(MAX_COUNTDOWN_SECS + 1)),
        None
    );
    assert!(pacing_from_parts("countdown", Some(MIN_COUNTDOWN_SECS)).is_some());
    assert!(pacing_from_parts("countdown", Some(MAX_COUNTDOWN_SECS)).is_some());
    assert_eq!(pacing_from_parts("stopwatch", None), None);
    assert_eq!(
        pacing_from_parts("free", Some(5)),
        Some(Pacing::Free),
        "seconds are ignored when they do not apply"
    );
}

#[test]
fn only_a_countdown_has_a_limit_in_milliseconds() {
    assert_eq!(limit_ms(Pacing::Countdown(90)), Some(90_000));
    assert_eq!(limit_ms(Pacing::Free), None);
    assert_eq!(limit_ms(Pacing::CountUp), None);
    assert_eq!(countdown_secs(Pacing::CountUp), None);
}
