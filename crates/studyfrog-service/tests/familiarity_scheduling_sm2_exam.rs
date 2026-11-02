#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity in scheduling: SM-2 and the exam: the same factor, and a stretched interval stops at an exam.

mod fam_support;

use fam_support::{DAY, Opts, chain, off, on, vault};
use studyfrog_core::review::Rating::Good;

#[test]
fn sm2_uses_the_same_factor_and_keeps_its_stored_interval() {
    let ratings = [Good, Good, Good, Good, Good, Good];
    let o = Opts {
        sm2: true,
        ..Opts::default()
    };
    let toml = Some("scheduler = \"sm2\"\n");
    let plain = chain(&vault("sm-off", toml), &ratings, &off(), &o);
    let with = chain(&vault("sm-on", toml), &ratings, &on(), &o);
    assert_eq!(plain.intervals[..2], with.intervals[..2]);
    assert_ne!(plain.intervals, with.intervals);
    assert_eq!(
        plain.stored, with.stored,
        "SM-2 keeps its own interval, so its progression is the same"
    );
}

#[test]
fn views_can_put_familiarity_ahead_and_a_stretched_interval_stops_at_an_exam() {
    let ratings = [Good, Good, Good, Good];
    let o = Opts {
        views: true,
        ..Opts::default()
    };
    let plain = chain(&vault("x-off", None), &ratings, &off(), &o);
    let free = chain(&vault("x-free", None), &ratings, &on(), &o);
    let at = 2;
    assert!(
        free.intervals[at] > plain.intervals[at],
        "sixty views put familiarity ahead of the stability: {:?} {:?}",
        free.intervals,
        plain.intervals
    );
    let wanted = (plain.intervals[at] + free.intervals[at]) / 2;
    let exam = free.starts[at] + wanted * DAY;
    let capped = chain(
        &vault("x-cap", None),
        &ratings,
        &on(),
        &Opts {
            exam_at: Some(exam),
            ..o
        },
    );
    let got = capped.intervals[at];
    assert!(
        got < free.intervals[at] && got >= plain.intervals[at],
        "{got} between {} and {}",
        plain.intervals[at],
        free.intervals[at]
    );
    assert!(
        (got - wanted).abs() <= 1,
        "cut at the exam: {got} vs {wanted}"
    );
    let soon = free.starts[at] + DAY;
    let floor = chain(
        &vault("x-soon", None),
        &ratings,
        &on(),
        &Opts {
            exam_at: Some(soon),
            ..o
        },
    );
    assert_eq!(
        floor.intervals[at], plain.intervals[at],
        "an exam tomorrow does not shorten below the plain interval"
    );
}
