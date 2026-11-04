#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cognitive_complexity)] // tests may panic on failure
//! Tests for self-assessment calibration against measured review performance.

use studyfrog_core::civil::Date;
use studyfrog_core::review::Rating;
use studyfrog_report::calibration::{
    CalibrationBias, CalibrationReview, bias_explanation, bias_label, calibration_report,
    calibration_review, diary_point, entry_score, normalize_rating,
};

#[test]
fn normalize_rating_scales_one_to_five_linearly() {
    assert_eq!(normalize_rating(1), 0.0);
    assert_eq!(normalize_rating(2), 0.25);
    assert_eq!(normalize_rating(3), 0.50);
    assert_eq!(normalize_rating(4), 0.75);
    assert_eq!(normalize_rating(5), 1.00);
}

#[test]
fn entry_score_averages_available_calibration_scales() {
    let p = diary_point(
        Date {
            year: 2026,
            month: 11,
            day: 4,
        },
        &[("performance", 4), ("focus", 4), ("energy", 2)],
    );
    // (0.75 + 0.75 + 0.25) / 3 = 1.75 / 3 = 0.5833333
    let score = entry_score(&p.ratings).expect("score should exist");
    assert!((score - 0.5833).abs() < 0.001);
}

#[test]
fn test_on_a_simulated_history_shows_the_calibration_measure_is_computed_correctly() {
    // 5-day simulated history:
    // Day 1: Self rating = 4 (0.75), 10 reviews with 8 recalled (80% accuracy). Fam = 0.60
    // Day 2: Self rating = 3 (0.50), 10 reviews with 5 recalled (50% accuracy). Fam = 0.55
    // Day 3: Self rating = 5 (1.00), 10 reviews with 9 recalled (90% accuracy). Fam = 0.85
    // Day 4: Self rating = 4 (0.75), 10 reviews with 7 recalled (70% accuracy). Fam = 0.70
    // Day 5: Self rating = 2 (0.25), 10 reviews with 3 recalled (30% accuracy). Fam = 0.35

    let entries = vec![
        diary_point(
            Date {
                year: 2026,
                month: 11,
                day: 1,
            },
            &[("performance", 4)],
        ),
        diary_point(
            Date {
                year: 2026,
                month: 11,
                day: 2,
            },
            &[("performance", 3)],
        ),
        diary_point(
            Date {
                year: 2026,
                month: 11,
                day: 3,
            },
            &[("performance", 5)],
        ),
        diary_point(
            Date {
                year: 2026,
                month: 11,
                day: 4,
            },
            &[("performance", 4)],
        ),
        diary_point(
            Date {
                year: 2026,
                month: 11,
                day: 5,
            },
            &[("performance", 2)],
        ),
    ];

    let mut reviews: Vec<CalibrationReview> = Vec::new();
    let sim_days = [
        (1, 8, 2, 0.60),
        (2, 5, 5, 0.55),
        (3, 9, 1, 0.85),
        (4, 7, 3, 0.70),
        (5, 3, 7, 0.35),
    ];

    for (day, good, again, fam) in sim_days {
        let date = Date {
            year: 2026,
            month: 11,
            day,
        };
        for _ in 0..good {
            reviews.push(calibration_review(date, Rating::Good, Some(fam)));
        }
        for _ in 0..again {
            reviews.push(calibration_review(date, Rating::Again, Some(fam)));
        }
    }

    let report = calibration_report(&entries, &reviews);

    // Verify sample size counts
    assert_eq!(report.entries_used, 5);
    assert_eq!(report.reviews_evaluated, 50);
    assert_eq!(report.buckets.len(), 5);

    // Day 1: self 0.75, acc 0.80 -> gap -0.05
    // Day 2: self 0.50, acc 0.50 -> gap 0.00
    // Day 3: self 1.00, acc 0.90 -> gap +0.10
    // Day 4: self 0.75, acc 0.70 -> gap +0.05
    // Day 5: self 0.25, acc 0.30 -> gap -0.05
    let b1 = &report.buckets[0];
    assert_eq!(b1.entries, 1);
    assert_eq!(b1.reviews, 10);
    assert!((b1.self_score.unwrap() - 0.75).abs() < 1e-4);
    assert!((b1.accuracy.unwrap() - 0.80).abs() < 1e-4);
    assert!((b1.familiarity.unwrap() - 0.60).abs() < 1e-4);
    assert!((b1.gap.unwrap() - (-0.05)).abs() < 1e-4);

    // Overall mean self score = (0.75 + 0.50 + 1.00 + 0.75 + 0.25) / 5 = 0.65
    assert!((report.mean_self_score.unwrap() - 0.65).abs() < 1e-4);

    // Overall mean accuracy = (8 + 5 + 9 + 7 + 3) / 50 = 32 / 50 = 0.64
    assert!((report.mean_accuracy.unwrap() - 0.64).abs() < 1e-4);

    // Overall mean familiarity = (10*0.60 + 10*0.55 + 10*0.85 + 10*0.70 + 10*0.35) / 50 = 0.61
    assert!((report.mean_familiarity.unwrap() - 0.61).abs() < 1e-4);

    // Mean delta = (-0.05 + 0.00 + 0.10 + 0.05 - 0.05) / 5 = 0.05 / 5 = +0.01
    assert!((report.delta.unwrap() - 0.01).abs() < 1e-4);

    // Mean absolute error = (0.05 + 0.00 + 0.10 + 0.05 + 0.05) / 5 = 0.25 / 5 = 0.05
    // Calibration score = 1.0 - 0.05 = 0.95
    assert!((report.score.unwrap() - 0.95).abs() < 1e-4);

    // Bias is within [-0.05, 0.05] -> WellCalibrated
    assert_eq!(report.bias, CalibrationBias::WellCalibrated);
    assert_eq!(bias_label(report.bias), "Accurately calibrated");
    assert!(!bias_explanation(report.bias).is_empty());
}

#[test]
fn simulated_overconfident_history_detects_overconfidence_bias() {
    let date = Date {
        year: 2026,
        month: 11,
        day: 4,
    };
    // User rates 5 (1.00), but only 2 of 10 items recalled (20% accuracy).
    let entries = vec![diary_point(date, &[("performance", 5)])];
    let mut reviews = Vec::new();
    for _ in 0..2 {
        reviews.push(calibration_review(date, Rating::Good, Some(0.3)));
    }
    for _ in 0..8 {
        reviews.push(calibration_review(date, Rating::Again, Some(0.2)));
    }

    let report = calibration_report(&entries, &reviews);
    assert_eq!(report.entries_used, 1);
    assert_eq!(report.reviews_evaluated, 10);
    assert_eq!(report.mean_self_score, Some(1.0));
    assert_eq!(report.mean_accuracy, Some(0.2));
    assert_eq!(report.delta, Some(0.8));
    assert_eq!(report.bias, CalibrationBias::Overconfident);
    assert_eq!(bias_label(report.bias), "Overconfident");
}

#[test]
fn simulated_underconfident_history_detects_underconfidence_bias() {
    let date = Date {
        year: 2026,
        month: 11,
        day: 4,
    };
    // User rates 1 (0.00), but 9 of 10 items recalled (90% accuracy).
    let entries = vec![diary_point(date, &[("performance", 1)])];
    let mut reviews = Vec::new();
    for _ in 0..9 {
        reviews.push(calibration_review(date, Rating::Good, Some(0.85)));
    }
    for _ in 0..1 {
        reviews.push(calibration_review(date, Rating::Again, Some(0.8)));
    }

    let report = calibration_report(&entries, &reviews);
    assert_eq!(report.entries_used, 1);
    assert_eq!(report.reviews_evaluated, 10);
    assert_eq!(report.mean_self_score, Some(0.0));
    assert_eq!(report.mean_accuracy, Some(0.9));
    assert!((report.delta.unwrap() - (-0.9)).abs() < 1e-4);
    assert_eq!(report.bias, CalibrationBias::Underconfident);
    assert_eq!(bias_label(report.bias), "Underconfident");
}

#[test]
fn empty_inputs_produce_safe_defaults() {
    let report = calibration_report(&[], &[]);
    assert_eq!(report.entries_used, 0);
    assert_eq!(report.reviews_evaluated, 0);
    assert_eq!(report.mean_self_score, None);
    assert_eq!(report.mean_accuracy, None);
    assert_eq!(report.mean_familiarity, None);
    assert_eq!(report.delta, None);
    assert_eq!(report.score, None);
    assert_eq!(report.bias, CalibrationBias::WellCalibrated);
    assert!(report.buckets.is_empty());
}
