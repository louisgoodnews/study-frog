#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the German grade scale: band edges, pass mark, labels, and exam summaries.

use studyfrog_rehearsal::grade::{
    GradeBadge, exam_result, grade_label, is_passing, percentage, percentage_to_german_grade,
};

fn label(pct: f32) -> String {
    grade_label(percentage_to_german_grade(pct, 50.0).0)
}

#[test]
fn band_edges_map_to_the_german_scale() {
    assert_eq!(label(100.0), "1.0");
    assert_eq!(label(95.0), "1.0");
    assert_eq!(label(94.9), "1.3");
    assert_eq!(label(90.0), "1.3");
    assert_eq!(label(85.0), "1.7");
    assert_eq!(label(80.0), "2.0");
    assert_eq!(label(75.0), "2.3");
    assert_eq!(label(70.0), "2.7");
    assert_eq!(label(65.0), "3.0");
    assert_eq!(label(60.0), "3.3");
    assert_eq!(label(55.0), "3.7");
    assert_eq!(label(50.0), "4.0");
    assert_eq!(label(49.9), "5.0");
    assert_eq!(label(0.0), "5.0");
}

#[test]
fn words_follow_the_bands() {
    assert_eq!(percentage_to_german_grade(82.0, 50.0).1, "Gut");
    assert_eq!(percentage_to_german_grade(40.0, 50.0).1, "Nicht bestanden");
}

#[test]
fn pass_mark_above_a_band_forces_a_fail() {
    let (badge, word) = percentage_to_german_grade(58.0, 60.0);
    assert_eq!(grade_label(badge), "5.0");
    assert_eq!(word, "Nicht bestanden");
}

#[test]
fn passing_means_four_or_better() {
    assert!(is_passing(GradeBadge { tenths: 40 }));
    assert!(!is_passing(GradeBadge { tenths: 50 }));
}

#[test]
fn percentage_caps_at_the_possible_points() {
    assert_eq!(percentage(41, 50), Some(82.0));
    assert_eq!(percentage(60, 50), Some(100.0));
    assert_eq!(percentage(0, 0), None);
}

#[test]
fn exam_result_sums_points_and_grades() {
    let r = exam_result(&[2, 5, 8, 4], &[2, 5, 8, 5], 50.0);
    assert_eq!(r.earned, 19);
    assert_eq!(r.possible, 20);
    assert_eq!(r.percent, Some(95.0));
    assert_eq!(grade_label(r.grade), "1.0");
}

#[test]
fn an_empty_exam_grades_as_a_fail() {
    let r = exam_result(&[], &[], 50.0);
    assert_eq!(r.percent, None);
    assert_eq!(grade_label(r.grade), "5.0");
}

#[test]
fn questions_are_summed_by_their_points_not_averaged_by_their_percentages() {
    // 1 of 3 points and 10 of 10 points: 11 of 13 is 84.6 % (grade 2.0). Averaging the two shares
    // would give 66.7 % (grade 3.0).
    let r = exam_result(&[1, 10], &[3, 10], 50.0);
    assert_eq!(grade_label(r.grade), "2.0");
    assert!((r.percent.unwrap() - 84.615).abs() < 0.01);
}

#[test]
fn a_configured_pass_mark_applies_to_the_whole_exam() {
    // 21 of 30 points is 70 %: grade 2.7 with a pass mark of 60, a fail (5.0) with 75.
    assert_eq!(grade_label(exam_result(&[21], &[30], 60.0).grade), "2.7");
    assert_eq!(grade_label(exam_result(&[21], &[30], 75.0).grade), "5.0");
}
