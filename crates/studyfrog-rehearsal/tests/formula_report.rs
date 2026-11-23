#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the drill result: standard scores of a raw score and the confidence band (EPIC-062).

use studyfrog_rehearsal::formula_report::score_report;

const TOL: f64 = 1e-9;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < TOL
}

#[test]
fn a_raw_score_at_the_mean_is_z_zero_and_the_usual_centres() {
    let r = score_report(50.0, 50.0, 10.0, 3.0);
    assert!(close(r.z, 0.0));
    assert!(close(r.t_score, 50.0));
    assert!(close(r.iq_score, 100.0));
}

#[test]
fn one_standard_deviation_above_the_mean_is_z_one() {
    let r = score_report(60.0, 50.0, 10.0, 3.0);
    assert!(close(r.z, 1.0));
    assert!(close(r.t_score, 60.0));
    assert!(close(r.iq_score, 115.0));
}

#[test]
fn the_band_is_the_raw_score_plus_or_minus_z_times_se() {
    // z for 95 % two-sided is 1.960, so the half width is 1.96 * 3.5 = 6.86.
    let r = score_report(60.0, 50.0, 10.0, 3.5);
    assert!(close(r.band.0, 60.0 - 1.96 * 3.5));
    assert!(close(r.band.1, 60.0 + 1.96 * 3.5));
}
