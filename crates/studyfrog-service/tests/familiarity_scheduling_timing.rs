#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Answer speed and the distance to an exam count in scheduling.

mod fam_support;

use fam_support::{DAY, Opts, chain, on, vault};
use studyfrog_core::review::Rating::Good;
use studyfrog_store::schedule::load_reviews_timed;

const RATINGS: [studyfrog_core::review::Rating; 5] = [Good, Good, Good, Good, Good];

#[test]
fn an_answer_read_faster_than_expected_comes_back_later_than_one_read_slowly() {
    let fast = Opts {
        timing: Some((1_000, 1_500)),
        ..Opts::default()
    };
    let slow = Opts {
        timing: Some((3_000, 1_500)),
        ..Opts::default()
    };
    let unknown = Opts::default();
    let (f, s, u) = (
        chain(&vault("speed-fast", None), &RATINGS, &on(), &fast),
        chain(&vault("speed-slow", None), &RATINGS, &on(), &slow),
        chain(&vault("speed-unknown", None), &RATINGS, &on(), &unknown),
    );
    assert_eq!(
        f.intervals[..2],
        s.intervals[..2],
        "nothing is touched before the third review"
    );
    let at = 3;
    assert!(
        f.intervals[at] > u.intervals[at] && u.intervals[at] > s.intervals[at],
        "fast {:?}, unknown {:?}, slow {:?}",
        f.intervals,
        u.intervals,
        s.intervals
    );
}

#[test]
fn the_timing_of_each_rating_is_kept_with_its_review() {
    let dir = vault("speed-kept", None);
    let o = Opts {
        timing: Some((1_200, 3_400)),
        ..Opts::default()
    };
    chain(&dir, &RATINGS[..3], &on(), &o);
    let (session, _) = studyfrog_service::session::open_vault(dir.0.to_str().unwrap()).unwrap();
    let reviews = load_reviews_timed(&session.conn, fam_support::ITEM).unwrap();
    assert_eq!(reviews.len(), 3);
    assert!(reviews.iter().all(|(_, t)| *t == Some((1_200, 3_400))));
}

#[test]
fn a_review_close_to_an_exam_gains_less_familiarity_so_the_item_comes_back_sooner() {
    let base = chain(&vault("exam-free", None), &RATINGS, &on(), &Opts::default());
    let at = 3;
    let exam = base.starts[at] + 2 * DAY;
    let near = chain(
        &vault("exam-near", None),
        &RATINGS,
        &on(),
        &Opts {
            exam_at: Some(exam),
            ..Opts::default()
        },
    );
    assert_eq!(
        base.intervals[..at],
        near.intervals[..at],
        "earlier reviews were far from the exam"
    );
    assert!(
        near.intervals[at] < base.intervals[at],
        "{:?} against {:?}",
        near.intervals,
        base.intervals
    );
    let far = chain(
        &vault("exam-far", None),
        &RATINGS,
        &on(),
        &Opts {
            exam_at: Some(base.starts[at] + 200 * DAY),
            ..Opts::default()
        },
    );
    assert_eq!(
        far.intervals[..=at],
        base.intervals[..=at],
        "an exam far away changes nothing"
    );
}
