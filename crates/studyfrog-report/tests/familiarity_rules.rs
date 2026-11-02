#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity: views, notes, the pauses of the fall, edits and stacks.

use studyfrog_core::priority::Priority;
use studyfrog_core::review::Rating::{self, Easy, Good};
use studyfrog_core::time::UnixMs;
use studyfrog_report::familiarity::{
    FamContext, FamEvent, Familiarity, fam_params_default, familiarity, stack_familiarity,
};

const DAY: i64 = 86_400_000;

fn day(d: i64) -> UnixMs {
    UnixMs(d * DAY)
}

fn rating(d: i64, r: Rating, s: f32) -> FamEvent {
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
        (40, Good, 1.0),
        (60, Good, 1.0),
    ]
    .map(|(d, r, s)| rating(d, r, s))
    .to_vec()
}

#[test]
fn views_add_a_little_short_ones_are_ignored_and_one_counts_per_hour() {
    let view = |h: i64, ms: u32| FamEvent::View {
        at: UnixMs(h * 3_600_000),
        active_ms: ms,
        expected_ms: None,
    };
    let note = FamContext {
        is_note: true,
        ..plain()
    };
    let one = value(&[view(0, 5_000)], 0, &note);
    assert!((one - 0.02).abs() < 0.001);
    assert_eq!(
        value(&[view(0, 500)], 0, &note),
        0.0,
        "under a second is an accident"
    );
    let burst = [
        view(0, 5_000),
        view(0, 5_000),
        FamEvent::View {
            at: UnixMs(1_800_000),
            active_ms: 5_000,
            expected_ms: None,
        },
    ];
    assert!(
        (value(&burst, 0, &note) - one).abs() < 1e-4,
        "within the hour only the first counts"
    );
    let spread = [view(0, 5_000), view(1, 5_000), view(2, 5_000)];
    assert!(value(&spread, 2, &note) > one);
}

#[test]
fn a_note_is_capped_and_its_confidence_rests_on_views() {
    let view = |d: i64| FamEvent::View {
        at: day(d),
        active_ms: 5_000,
        expected_ms: Some(1_000_000),
    };
    let many: Vec<FamEvent> = (0..400).map(|d| view(d / 200 + d)).collect();
    let note = FamContext {
        is_note: true,
        ..plain()
    };
    let got = familiarity(&many, day(500), &note, &fam_params_default()).unwrap();
    assert!(got.value <= 0.8 + f32::EPSILON);
    assert!(
        (got.confidence - 0.5).abs() < f32::EPSILON,
        "a note never gets more than half"
    );
    let card = familiarity(&ten_reviews(), day(260), &plain(), &fam_params_default()).unwrap();
    assert_eq!(card.confidence, 1.0);
    let one = familiarity(
        &[rating(0, Good, 1.0)],
        day(0),
        &plain(),
        &fam_params_default(),
    )
    .unwrap();
    assert!((one.confidence - 0.2).abs() < f32::EPSILON);
}

#[test]
fn the_fall_pauses_for_a_finished_item_and_after_the_exam() {
    let events = [rating(0, Easy, 1.0)];
    let free = value(&events, 365, &plain());
    assert!((free - 0.225).abs() < 0.001, "half after a year");
    assert!(
        (value(
            &events,
            365,
            &FamContext {
                done: true,
                ..plain()
            }
        ) - 0.45)
            .abs()
            < 0.001
    );
    let exam_at = |d: i64| FamContext {
        exam: Some(day(d)),
        ..plain()
    };
    assert!(
        (value(&events, 365, &exam_at(0)) - 0.45).abs() < 0.001,
        "the exam was the day of the last contact"
    );
    assert!(
        (value(&events, 365, &exam_at(365)) - free).abs() < 0.001,
        "an exam today still lets it fall"
    );
    assert!(
        (value(&events, 365, &exam_at(10_000)) - free).abs() < 0.001,
        "an exam in the future changes nothing"
    );
    let half = value(&events, 365, &exam_at(182));
    assert!(
        half > free && half < 0.45,
        "it falls only until the exam: {half}"
    );
}

#[test]
fn no_history_has_no_value_and_an_edit_changes_nothing_under_an_exponential_fall() {
    assert!(familiarity(&[], day(0), &plain(), &fam_params_default()).is_none());
    let base = [rating(0, Good, 1.0)];
    let mut edited = base.to_vec();
    edited.push(FamEvent::Edit { at: day(300) });
    let (a, b) = (value(&base, 365, &plain()), value(&edited, 365, &plain()));
    assert!(
        (a - b).abs() < 1e-5,
        "an edit is kept as an event but neither raises nor resets anything: {a} {b}"
    );
}

#[test]
fn a_stack_is_the_priority_weighted_mean_of_its_items() {
    let f = |v: f32, c: f32| Familiarity {
        value: v,
        confidence: c,
    };
    assert!(stack_familiarity(&[]).is_none());
    let got = stack_familiarity(&[
        (Priority::Highest, f(1.0, 1.0)),
        (Priority::Lowest, f(0.0, 0.0)),
    ])
    .unwrap();
    assert!(
        (got.value - 0.8).abs() < 1e-6 && (got.confidence - 0.8).abs() < 1e-6,
        "weights 2 and 0.5"
    );
    let even = stack_familiarity(&[
        (Priority::Medium, f(0.2, 0.2)),
        (Priority::Medium, f(0.6, 1.0)),
    ])
    .unwrap();
    assert!((even.value - 0.4).abs() < 1e-6);
}
