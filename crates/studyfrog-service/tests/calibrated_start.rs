#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calibrated starting difficulty (STORY-0177): with no observations the scheduler's prior is
//! used; observed difficulties move it, in the direction they point.

mod fam_support;

use fam_support::{Opts, chain, on, vault};
use studyfrog_core::review::Rating::{Good, Hard};
use studyfrog_service::schedule::{calibrated_initial, observed_difficulties};
use studyfrog_service::session::open_vault;
use studyfrog_srs::fsrs::initial_state;
use studyfrog_srs::params::params_default;
use studyfrog_store::first_timing::first_review_timings;

#[test]
fn without_observations_the_prior_is_unchanged() {
    let p = params_default();
    for rating in [Hard, Good] {
        assert_eq!(
            calibrated_initial(rating, &p, &[]),
            initial_state(rating, &p)
        );
    }
}

#[test]
fn hard_observations_raise_the_starting_difficulty_and_easy_ones_lower_it() {
    let p = params_default();
    let prior = initial_state(Good, &p).difficulty;
    let harder = calibrated_initial(Good, &p, &[9.0; 20]).difficulty;
    let easier = calibrated_initial(Good, &p, &[1.5; 20]).difficulty;
    assert!(harder > prior, "{harder} vs {prior}");
    assert!(easier < prior, "{easier} vs {prior}");
}

#[test]
fn the_timings_read_are_one_per_item_its_first_review() {
    let dir = vault("calibrated-first", None);
    let opts = Opts {
        timing: Some((4_000, 3_000)),
        ..Default::default()
    };
    chain(&dir, &[Good, Good, Hard], &on(), &opts);
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let rows = first_review_timings(&session.conn).unwrap();
    assert_eq!(rows, vec![(4_000, 3_000)]);
    let observed = observed_difficulties(&session.conn).unwrap();
    assert_eq!(observed.len(), 1);
}
