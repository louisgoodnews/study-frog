#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Slow reads, calibrated speeds, the user's mark and configurable thresholds.

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_rehearsal::reading::{
    Reason, expected_ms, measured_wpm, reading_params, reading_params_default, suggest,
    suggest_from_mark,
};

#[test]
fn a_slow_read_suggests_one_step_harder_but_never_softens_a_low_rating() {
    let p = reading_params_default();
    let slow = |rated, ms| suggest(20, ms, Some(Rating::Good), rated, &p);
    let good = slow(Rating::Good, 12_000).unwrap();
    assert_eq!((good.rating, good.reason), (Rating::Hard, Reason::Slow));
    assert_eq!(slow(Rating::Easy, 12_000).unwrap().rating, Rating::Good);
    assert_eq!(slow(Rating::Hard, 12_000), None);
    assert_eq!(slow(Rating::Again, 12_000), None);
    assert_eq!(
        suggest(7, 60_000, None, Rating::Good, &p),
        None,
        "short texts say nothing"
    );
    assert_eq!(
        suggest(20, 12_000, None, Rating::Good, &p).unwrap().reason,
        Reason::Slow,
        "new or not"
    );
}

#[test]
fn a_measured_speed_moves_the_baseline() {
    let slow_reader = reading_params(Some(120), &Default::default());
    assert_eq!(expected_ms(20, &slow_reader), 10_000);
    assert_eq!(
        suggest(20, 5000, Some(Rating::Good), Rating::Good, &slow_reader)
            .unwrap()
            .rating,
        Rating::Easy
    );
    assert_eq!(
        suggest(
            20,
            9000,
            Some(Rating::Good),
            Rating::Good,
            &reading_params(None, &Default::default())
        ),
        None
    );
    assert_eq!(
        reading_params(Some(5), &Default::default()).words_per_minute,
        50,
        "clamped"
    );
    assert_eq!(
        reading_params(None, &Default::default()),
        reading_params_default()
    );
}

#[test]
fn a_timed_reading_gives_words_per_minute_only_when_plausible() {
    assert_eq!(measured_wpm(120, 30_000), Some(240));
    assert_eq!(measured_wpm(120, 0), None);
    assert_eq!(measured_wpm(0, 30_000), None);
    assert_eq!(measured_wpm(120, 1_000), None, "7200 wpm is not reading");
    assert_eq!(measured_wpm(10, 600_000), None, "1 wpm is not reading");
}

#[test]
fn the_users_mark_suggests_a_rating_but_never_overrides_a_low_one() {
    let from = |m, r| suggest_from_mark(Some(m), r).map(|s| (s.rating, s.reason));
    assert_eq!(
        from(Difficulty::Difficult, Rating::Good),
        Some((Rating::Hard, Reason::MarkedDifficult))
    );
    assert_eq!(
        from(Difficulty::Difficult, Rating::Easy),
        Some((Rating::Hard, Reason::MarkedDifficult))
    );
    assert_eq!(
        from(Difficulty::Easy, Rating::Good),
        Some((Rating::Easy, Reason::MarkedEasy))
    );
    assert_eq!(
        from(Difficulty::Easy, Rating::Hard),
        None,
        "they struggled, whatever they felt"
    );
    assert_eq!(from(Difficulty::Easy, Rating::Easy), None);
    assert_eq!(from(Difficulty::Difficult, Rating::Again), None);
    assert_eq!(from(Difficulty::Medium, Rating::Good), None);
    assert_eq!(suggest_from_mark(None, Rating::Good), None);
}

#[test]
fn threshold_overrides_change_what_counts_as_fast_slow_and_long_enough() {
    use studyfrog_core::settings::ReadingTuning;
    let strict = reading_params(
        None,
        &ReadingTuning {
            fast_percent: Some(50),
            slow_percent: Some(300),
            min_words: Some(30),
        },
    );
    assert_eq!(
        (strict.fast_percent, strict.slow_percent, strict.min_words),
        (50, 300, 30)
    );
    assert_eq!(
        suggest(20, 3000, Some(Rating::Good), Rating::Good, &strict),
        None,
        "20 words are too few now"
    );
    let lenient = reading_params(
        None,
        &ReadingTuning {
            min_words: Some(1),
            ..Default::default()
        },
    );
    assert!(suggest(5, 100, Some(Rating::Good), Rating::Good, &lenient).is_some());
    assert_eq!(
        reading_params(None, &ReadingTuning::default()),
        reading_params_default()
    );
}
