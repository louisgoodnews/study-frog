//! Types and helpers for self-assessment calibration.

use std::collections::BTreeMap;

use studyfrog_core::civil::Date;
use studyfrog_core::diary::{DiaryEntry, RatingValue, rating_standard_score};
use studyfrog_core::review::Rating;

/// Priority rating keys considered for subjective self-assessment.
pub const CALIBRATION_SCALES: [&str; 6] = [
    "performance",
    "understanding",
    "confidence",
    "satisfaction",
    "focus",
    "energy",
];

/// Systematic cognitive bias observed between self-ratings and objective performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationBias {
    /// Subjective ratings align closely with measured recall (gap within +/- 0.05).
    WellCalibrated,
    /// Subjective ratings exceed measured recall (gap > +0.05).
    Overconfident,
    /// Subjective ratings are lower than measured recall (gap < -0.05).
    Underconfident,
}

/// A diary entry's date and ratings reduced to what calibration needs.
#[derive(Debug, Clone, PartialEq)]
pub struct DiaryRatingPoint {
    /// Civil date of the entry.
    pub date: Date,
    /// Self-assessment ratings.
    pub ratings: BTreeMap<String, RatingValue>,
}

/// A review event with its date, rating and optional item familiarity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalibrationReview {
    /// Civil date of the review.
    pub date: Date,
    /// SRS rating given (Again, Hard, Good, Easy).
    pub rating: Rating,
    /// Item familiarity at the time of review, if known.
    pub familiarity: Option<f32>,
}

/// A time-series bucket comparing subjective and objective metrics on a given date.
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationBucket {
    /// Date of the bucket.
    pub date: Date,
    /// Number of diary entries for this date.
    pub entries: usize,
    /// Number of reviews completed on this date.
    pub reviews: usize,
    /// Mean self-rated score (0.0 ..= 1.0), or None if no ratings.
    pub self_score: Option<f32>,
    /// Measured accuracy (% recalled without Again, 0.0 ..= 1.0).
    pub accuracy: Option<f32>,
    /// Average item familiarity on this date (0.0 ..= 1.0), if known.
    pub familiarity: Option<f32>,
    /// Subjective score minus objective accuracy (self_score - accuracy), if both exist.
    pub gap: Option<f32>,
}

/// Overall calibration report summarizing self-assessment against measured results.
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationReport {
    /// Total number of diary entries used in this report.
    pub entries_used: usize,
    /// Total number of reviews evaluated.
    pub reviews_evaluated: usize,
    /// Average self-rated score across evaluated entries (0.0 ..= 1.0).
    pub mean_self_score: Option<f32>,
    /// Overall review accuracy (% recalled without Again, 0.0 ..= 1.0).
    pub mean_accuracy: Option<f32>,
    /// Average item familiarity across reviews (0.0 ..= 1.0).
    pub mean_familiarity: Option<f32>,
    /// Calibration delta (subjective - objective gap).
    pub delta: Option<f32>,
    /// Calibration score: alignment measure (1.0 - mean absolute error, clamped 0.0 ..= 1.0).
    pub score: Option<f32>,
    /// Direction and type of cognitive bias.
    pub bias: CalibrationBias,
    /// Daily buckets ordered chronologically.
    pub buckets: Vec<CalibrationBucket>,
}

/// Normalize a 1..=5 standard scale rating to 0.0..=1.0.
pub fn normalize_rating(rating: u8) -> f32 {
    (f32::from(rating.clamp(1, 5)) - 1.0) / 4.0
}

/// Create a [`DiaryRatingPoint`] from a slice of (key, score) tuples.
pub fn diary_point(date: Date, ratings: &[(&str, i64)]) -> DiaryRatingPoint {
    let mut map = BTreeMap::new();
    for (k, v) in ratings {
        map.insert((*k).to_owned(), RatingValue::Int(*v));
    }
    DiaryRatingPoint { date, ratings: map }
}

/// Create a [`DiaryRatingPoint`] from a [`DiaryEntry`].
pub fn diary_point_from_entry(e: &DiaryEntry) -> DiaryRatingPoint {
    DiaryRatingPoint {
        date: e.date,
        ratings: e.ratings.clone(),
    }
}

/// Create a [`CalibrationReview`].
pub fn calibration_review(
    date: Date,
    rating: Rating,
    familiarity: Option<f32>,
) -> CalibrationReview {
    CalibrationReview {
        date,
        rating,
        familiarity,
    }
}

/// Extract the normalized subjective performance score (0.0..=1.0) from a ratings map.
pub fn entry_score(ratings: &BTreeMap<String, RatingValue>) -> Option<f32> {
    let mut scores = Vec::new();
    for key in CALIBRATION_SCALES {
        if let Some(val) = ratings.get(key).and_then(rating_standard_score) {
            scores.push(normalize_rating(val));
        }
    }
    if scores.is_empty() {
        for val in ratings.values().filter_map(rating_standard_score) {
            scores.push(normalize_rating(val));
        }
    }
    if scores.is_empty() {
        None
    } else {
        let sum: f32 = scores.iter().sum();
        Some(sum / scores.len() as f32)
    }
}

/// Human-readable label for a [`CalibrationBias`].
pub fn bias_label(bias: CalibrationBias) -> &'static str {
    match bias {
        CalibrationBias::WellCalibrated => "Accurately calibrated",
        CalibrationBias::Overconfident => "Overconfident",
        CalibrationBias::Underconfident => "Underconfident",
    }
}

/// Explanatory guidance for a [`CalibrationBias`].
pub fn bias_explanation(bias: CalibrationBias) -> &'static str {
    match bias {
        CalibrationBias::WellCalibrated => {
            "Your self-ratings closely match your measured recall performance."
        }
        CalibrationBias::Overconfident => {
            "You tend to rate your grasp higher than your measured review accuracy."
        }
        CalibrationBias::Underconfident => {
            "You tend to underestimate your grasp; measured recall is higher than your self-ratings."
        }
    }
}
