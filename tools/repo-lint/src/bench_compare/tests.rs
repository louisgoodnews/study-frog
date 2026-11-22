#![allow(clippy::unwrap_used, clippy::expect_used, clippy::float_cmp)]

use super::*;

const TIGHT: [f64; 5] = [100.0, 101.0, 99.0, 100.0, 100.0];

fn scaled(xs: &[f64], k: f64) -> Vec<f64> {
    xs.iter().map(|x| x * k).collect()
}

#[test]
fn identical_runs_are_no_change() {
    let c = compare(&TIGHT, &TIGHT).unwrap();
    assert_eq!(c.verdict, Verdict::NoChange);
    assert!((c.ratio - 1.0).abs() < 1e-12);
}

#[test]
fn twenty_percent_slower_with_tight_runs_is_a_regression() {
    let new = scaled(&TIGHT, 1.2);
    let c = compare(&TIGHT, &new).unwrap();
    assert_eq!(c.verdict, Verdict::Regression);
    assert!(c.ci_low > 1.0);
}

#[test]
fn three_percent_slower_is_below_the_floor() {
    let new = scaled(&TIGHT, 1.03);
    let c = compare(&TIGHT, &new).unwrap();
    assert!(c.ci_low > 1.0, "the interval alone would flag it");
    assert_eq!(c.verdict, Verdict::NoChange);
}

#[test]
fn noisy_baseline_raises_the_floor_and_hides_a_twenty_percent_change() {
    let noisy = [100.0, 130.0, 80.0, 115.0, 90.0];
    let new = scaled(&TIGHT, 1.2);
    let c = compare(&noisy, &new).unwrap();
    assert!(c.floor > 0.2, "floor {} should be 3x the spread", c.floor);
    assert_eq!(c.verdict, Verdict::NoChange);
}

#[test]
fn twenty_percent_faster_is_an_improvement_not_a_regression() {
    let new = scaled(&TIGHT, 0.8);
    let c = compare(&TIGHT, &new).unwrap();
    assert_eq!(c.verdict, Verdict::Improvement);
}

#[test]
fn one_outlier_run_does_not_flag_a_regression() {
    let base = [100.0; 5];
    let new = [100.0, 100.0, 100.0, 100.0, 300.0];
    let c = compare(&base, &new).unwrap();
    assert_eq!(c.verdict, Verdict::NoChange);
    assert!((c.ratio - 1.0).abs() < 1e-12);
}

#[test]
fn bootstrap_is_deterministic() {
    assert_eq!(bootstrap_ci(&TIGHT, &TIGHT), bootstrap_ci(&TIGHT, &TIGHT));
}

#[test]
fn spread_and_repeat_rule() {
    assert!((spread(&[100.0, 110.0]) - 10.0 / 105.0).abs() < 1e-12);
    assert!(needs_ten_runs(&[100.0, 100.0, 100.0, 100.0, 110.0]));
    assert!(!needs_ten_runs(&TIGHT));
    let ten = [
        100.0, 110.0, 100.0, 110.0, 100.0, 110.0, 100.0, 110.0, 100.0, 110.0,
    ];
    assert!(!needs_ten_runs(&ten), "ten runs are already enough");
}

#[test]
fn parsing_skips_comments_and_rejects_bad_values() {
    assert_eq!(parse_samples("# c\n1.5\n\n2\n").unwrap(), vec![1.5, 2.0]);
    assert!(parse_samples("abc").is_err());
    assert!(parse_samples("-3").is_err());
    assert!(parse_samples("0").is_err());
}

#[test]
fn a_noisy_side_asks_for_ten_runs() {
    let noisy = [100.0, 110.0, 100.0, 110.0, 150.0];
    let c = compare(&noisy, &TIGHT).unwrap();
    assert!(c.more_runs_needed);
    assert!(!compare(&TIGHT, &TIGHT).unwrap().more_runs_needed);
}

#[test]
fn empty_side_is_an_error() {
    assert!(compare(&[], &TIGHT).is_err());
}

#[test]
fn line_names_the_verdict_and_the_numbers() {
    let c = compare(&TIGHT, &scaled(&TIGHT, 1.2)).unwrap();
    let line = format_line(&c);
    assert!(line.starts_with("verdict=regression ratio=1.200"), "{line}");
    assert!(line.ends_with("runs=5/5 more_runs_needed=no"), "{line}");
}
