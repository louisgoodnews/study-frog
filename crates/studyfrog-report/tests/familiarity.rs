#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity: the worked examples of docs/specs/familiarity.md, the rules around them and the properties.

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating::{self, Again, Easy, Good, Hard};
use studyfrog_core::time::UnixMs;
use studyfrog_report::familiarity::{FamContext, FamEvent, fam_params_default, familiarity};

const DAY: i64 = 86_400_000;

fn day(d: i64) -> UnixMs {
    UnixMs(d * DAY)
}

fn rating(d: i64, r: Rating, s: f32) -> FamEvent {
    // `s` is the speed factor: expected / actual.
    FamEvent::Rating {
        at: day(d),
        rating: r,
        answer_ms: Some(1000),
        expected_ms: Some((1000.0 * s) as u32),
    }
}

fn value(events: &[FamEvent], now: i64, ctx: &FamContext) -> f32 {
    familiarity(events, day(now), ctx, &fam_params_default())
        .unwrap()
        .value
}

fn plain() -> FamContext {
    FamContext::default()
}

fn ten_reviews() -> Vec<FamEvent> {
    [
        (0, Good, 1.0),
        (2, Good, 1.0),
        (6, Easy, 1.5),
        (15, Good, 1.0),
        (40, Hard, 1.0),
        (43, Good, 1.0),
        (60, Good, 1.0),
        (110, Easy, 1.5),
        (170, Good, 1.0),
        (260, Easy, 1.5),
    ]
    .map(|(d, r, s)| rating(d, r, s))
    .to_vec()
}

#[test]
fn the_card_over_ten_reviews_follows_the_specification_table() {
    let all = ten_reviews();
    let after = [0.30, 0.51, 0.84, 0.88, 0.63, 0.74, 0.80, 0.91, 0.87, 0.91];
    for (i, want) in after.iter().enumerate() {
        let got = value(&all[..=i], all_day(&all[i]), &plain());
        assert!(
            (got - want).abs() < 0.011,
            "after review {}: {got} vs {want}",
            i + 1
        );
    }
    let end = value(&all, 260 + 90, &plain());
    assert!(
        (end - 0.77).abs() < 0.011,
        "after 90 days without contact: {end}"
    );
    assert!(
        (value(&all, 260 + 365, &plain()) - 0.46).abs() < 0.011,
        "after a year"
    );
    let done = FamContext {
        done: true,
        ..plain()
    };
    assert!(
        (value(&all, 260 + 365, &done) - 0.91).abs() < 0.011,
        "a finished item stays"
    );
    let mut lapse = all.clone();
    lapse.push(rating(300, Again, 1.0));
    assert!((value(&lapse, 300, &plain()) - 0.42).abs() < 0.011);
}

fn all_day(e: &FamEvent) -> i64 {
    match e {
        FamEvent::Rating { at, .. } | FamEvent::View { at, .. } | FamEvent::Edit { at } => {
            at.0 / DAY
        }
    }
}

#[test]
fn the_same_rating_is_worth_more_when_the_answer_was_read_faster() {
    let first = |r: Rating, s: f32| value(&[rating(0, r, s)], 0, &plain());
    for (got, want) in [
        (first(Easy, 1.5), 0.675),
        (first(Easy, 1.0), 0.45),
        (first(Easy, 0.5), 0.225),
        (first(Good, 1.5), 0.45),
        (first(Good, 1.0), 0.30),
        (first(Good, 0.5), 0.15),
    ] {
        assert!((got - want).abs() < 0.001, "{got} vs {want}");
    }
    let unknown = FamEvent::Rating {
        at: day(0),
        rating: Easy,
        answer_ms: None,
        expected_ms: None,
    };
    assert!(
        (value(&[unknown], 0, &plain()) - 0.45).abs() < 0.001,
        "no measurement counts as the expected time"
    );
}

#[test]
fn marks_scale_the_gains_but_not_the_losses() {
    let events = [rating(0, Good, 1.0)];
    let with = |m: Option<Difficulty>| value(&events, 0, &FamContext { mark: m, ..plain() });
    assert!((with(Some(Difficulty::Easy)) - 0.33).abs() < 0.001);
    assert!((with(Some(Difficulty::Difficult)) - 0.24).abs() < 0.001);
    assert!((with(Some(Difficulty::Medium)) - 0.30).abs() < 0.001);
    let lapse = [rating(0, Again, 1.0)];
    assert_eq!(
        value(
            &lapse,
            0,
            &FamContext {
                mark: Some(Difficulty::Difficult),
                ..plain()
            }
        ),
        0.0
    );
}
