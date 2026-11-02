#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity: properties that hold for any history.

use proptest::prelude::*;
use studyfrog_core::review::Rating::{self, Again, Easy, Good, Hard};
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

fn all_day(e: &FamEvent) -> i64 {
    match e {
        FamEvent::Rating { at, .. } | FamEvent::View { at, .. } | FamEvent::Edit { at } => {
            at.0 / DAY
        }
    }
}

fn rating_of(n: u8) -> Rating {
    [Again, Hard, Good, Easy][usize::from(n % 4)]
}

fn sequence(steps: &[(u8, u8)]) -> Vec<FamEvent> {
    let mut d = 0;
    steps
        .iter()
        .map(|(gap, r)| {
            d += i64::from(*gap);
            rating(d, rating_of(*r), 1.0)
        })
        .collect()
}

proptest! {
    #[test]
    fn the_value_stays_between_zero_and_one(steps in proptest::collection::vec((0_u8..90, 0_u8..4), 1..40), later in 0_i64..2000) {
        let events = sequence(&steps);
        let v = value(&events, all_day(events.last().unwrap()) + later, &plain());
        prop_assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn a_rating_moves_the_value_in_the_order_easy_good_prior_hard_again(steps in proptest::collection::vec((0_u8..90, 0_u8..4), 1..20)) {
        let base = sequence(&steps);
        let t = all_day(base.last().unwrap());
        let prior = value(&base, t, &plain());
        let with = |r: Rating| { let mut e = base.clone(); e.push(rating(t, r, 1.0)); value(&e, t, &plain()) };
        let (e, g, h, a) = (with(Easy), with(Good), with(Hard), with(Again));
        prop_assert!(e + 1e-6 >= g && g + 1e-6 >= prior && prior + 1e-6 >= h && h + 1e-6 >= a, "{e} {g} {prior} {h} {a}");
    }

    #[test]
    fn a_view_never_lowers_the_value(steps in proptest::collection::vec((0_u8..90, 0_u8..4), 1..20), ms in 1000_u32..600_000) {
        let base = sequence(&steps);
        let t = all_day(base.last().unwrap());
        let mut with = base.clone();
        with.push(FamEvent::View { at: UnixMs(t * DAY + 7_200_000), active_ms: ms, expected_ms: Some(5_000) });
        let after = familiarity(&with, UnixMs(t * DAY + 7_200_000), &plain(), &fam_params_default()).unwrap().value;
        let before = familiarity(&base, UnixMs(t * DAY + 7_200_000), &plain(), &fam_params_default()).unwrap().value;
        prop_assert!(after + 1e-6 >= before);
    }

    #[test]
    fn a_faster_answer_never_gives_a_lower_value(steps in proptest::collection::vec((0_u8..90, 0_u8..4), 1..20), slow in 500_u32..20_000, fast_by in 0_u32..400) {
        let mut a = sequence(&steps);
        let mut b = a.clone();
        let t = all_day(a.last().unwrap());
        let timed = |ms: u32| FamEvent::Rating { at: day(t + 1), rating: Good, answer_ms: Some(ms), expected_ms: Some(3_000) };
        a.push(timed(slow));
        b.push(timed(slow.saturating_sub(fast_by).max(1)));
        prop_assert!(value(&b, t + 1, &plain()) + 1e-6 >= value(&a, t + 1, &plain()));
    }

    #[test]
    fn without_contact_the_value_never_rises_and_stands_still_for_a_finished_item(steps in proptest::collection::vec((0_u8..90, 0_u8..4), 1..20), a in 0_i64..800, b in 0_i64..800) {
        let events = sequence(&steps);
        let t = all_day(events.last().unwrap());
        let (near, far) = (a.min(b), a.max(b));
        prop_assert!(value(&events, t + far, &plain()) <= value(&events, t + near, &plain()) + 1e-6);
        let done = FamContext { done: true, ..plain() };
        prop_assert!((value(&events, t + far, &done) - value(&events, t + near, &done)).abs() < 1e-6);
    }

    #[test]
    fn the_order_the_events_are_stored_in_does_not_matter(steps in proptest::collection::vec((0_u8..90, 0_u8..4), 1..20)) {
        let events = sequence(&steps);
        let mut reversed = events.clone();
        reversed.reverse();
        let t = all_day(events.last().unwrap());
        prop_assert_eq!(value(&events, t + 10, &plain()), value(&reversed, t + 10, &plain()));
    }
}
