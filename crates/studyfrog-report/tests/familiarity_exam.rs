#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity close to an exam or due date: gains count for less, losses and the fall are untouched.

use studyfrog_core::review::Rating::{self, Again, Good, Hard};
use studyfrog_core::time::UnixMs;
use studyfrog_report::familiarity::{FamContext, FamEvent, fam_params_default, familiarity};

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

#[test]
fn a_gain_close_to_an_exam_counts_for_less_and_one_far_from_it_or_after_it_counts_in_full() {
    let exam = |d: i64| FamContext {
        exam: Some(day(d)),
        ..plain()
    };
    let first = |at: i64, ctx: &FamContext| value(&[rating(at, Good, 1.0)], at, ctx);
    let free = first(0, &plain());
    assert!((free - 0.30).abs() < 1e-4);
    assert!(
        (first(0, &exam(100)) - free).abs() < 1e-4,
        "far from the exam"
    );
    assert!(
        (first(0, &exam(14)) - free).abs() < 1e-4,
        "fourteen days or more is far"
    );
    assert!(
        (first(0, &exam(7)) - 0.30 * 0.75).abs() < 1e-3,
        "a week before: three quarters of a gain"
    );
    assert!((first(0, &exam(1)) - 0.30 * (0.5 + 0.5 / 14.0)).abs() < 1e-3);
    let on_the_day = value(&[rating(10, Good, 1.0)], 10, &exam(10));
    assert!(
        (on_the_day - free).abs() < 1e-4,
        "an exam that is not ahead damps nothing"
    );
    let before = |d: i64| first(0, &exam(d));
    assert!(
        before(2) < before(5) && before(5) < before(9) && before(9) < before(14),
        "the nearer, the less"
    );
}

#[test]
fn losses_are_not_damped_by_an_exam_and_views_are() {
    // the exam is two days off: the Good at day 0 is damped, the later events are losses
    let exam = FamContext {
        exam: Some(day(2)),
        ..plain()
    };
    let good = value(&[rating(0, Good, 1.0)], 1, &exam);
    let lapsed = value(&[rating(0, Good, 1.0), rating(1, Again, 1.0)], 1, &exam);
    assert!(
        (lapsed / good - 0.5).abs() < 5e-3,
        "a lapse still takes half: {good} -> {lapsed}"
    );
    let harder = value(&[rating(0, Good, 1.0), rating(1, Hard, 1.0)], 1, &exam);
    assert!((harder / good - 0.75).abs() < 5e-3);
    let view = |ctx: &FamContext| {
        value(
            &[FamEvent::View {
                at: day(0),
                active_ms: 5_000,
                expected_ms: None,
            }],
            0,
            ctx,
        )
    };
    let note = |e: Option<UnixMs>| FamContext {
        is_note: true,
        exam: e,
        ..plain()
    };
    assert!(
        view(&note(Some(day(1)))) < view(&note(None)),
        "a view close to the exam counts for less too"
    );
}
