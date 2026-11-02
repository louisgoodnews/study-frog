#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity in scheduling: ratings put the due date where familiarity says, the stored state stays,
//! and the switches and the deadline are respected.

mod fam_support;

use fam_support::{Opts, chain, off, on, vault};
use studyfrog_core::review::Rating::{Easy, Good};

#[test]
fn a_strength_of_zero_is_plain_fsrs_and_nothing_happens_before_the_third_review() {
    let ratings = [Good, Good, Good, Good, Good, Good];
    let plain = chain(&vault("off", None), &ratings, &off(), &Opts::default());
    let with = chain(&vault("on", None), &ratings, &on(), &Opts::default());
    assert_eq!(
        plain.intervals[..2],
        with.intervals[..2],
        "reviews one and two are not touched"
    );
    assert_eq!(
        plain.stored[..3],
        with.stored[..3],
        "so the first three states agree"
    );
    assert_ne!(
        plain.intervals, with.intervals,
        "later reviews are put elsewhere"
    );
}

#[test]
fn ratings_whose_familiarity_lags_the_stability_come_back_sooner() {
    let ratings = [Easy, Easy, Easy, Easy, Easy];
    let plain = chain(&vault("easy-off", None), &ratings, &off(), &Opts::default());
    let with = chain(&vault("easy-on", None), &ratings, &on(), &Opts::default());
    let at = 3;
    assert!(
        with.intervals[at] < plain.intervals[at],
        "{:?} {:?}",
        with.intervals,
        plain.intervals
    );
}

#[test]
fn a_stack_can_switch_familiarity_off_and_on() {
    let ratings = [Good, Good, Good, Good, Good];
    let plain = chain(&vault("p", None), &ratings, &off(), &Opts::default());
    let switched = chain(
        &vault("sw", Some("familiarity = false\n")),
        &ratings,
        &on(),
        &Opts::default(),
    );
    assert_eq!(plain.intervals, switched.intervals);
    let forced = chain(
        &vault("ons", Some("familiarity = true\n")),
        &ratings,
        &on(),
        &Opts::default(),
    );
    assert_ne!(plain.intervals, forced.intervals);
}
