//! Retention by time since the previous review. Pure.

use studyfrog_core::review::Rating;

/// A review that followed an earlier one, reduced to what retention needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReviewPoint {
    /// The rating given.
    pub rating: Rating,
    /// Days since the previous review of the same item.
    pub elapsed_days: f32,
}

/// Interval buckets in days: (first, last), the last one open-ended.
pub const BUCKETS: [(u32, u32); 8] = [
    (1, 1),
    (2, 2),
    (3, 4),
    (5, 7),
    (8, 14),
    (15, 30),
    (31, 90),
    (91, u32::MAX),
];

/// How often items were recalled after a gap of this many days.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionBucket {
    /// Shortest gap in the bucket, in days.
    pub from_days: u32,
    /// Longest gap in the bucket, `u32::MAX` when open-ended.
    pub to_days: u32,
    /// Reviews in the bucket.
    pub reviews: u32,
    /// Of those, how many were not rated Again.
    pub recalled: u32,
}

/// Share of reviews that were recalled, or `None` for an empty bucket.
pub fn retention_rate(b: &RetentionBucket) -> Option<f32> {
    (b.reviews > 0).then(|| b.recalled as f32 / b.reviews as f32)
}

/// Retention per gap bucket. Reviews on the same day as the previous one (gap under a day) are left out.
pub fn retention_curve(points: &[ReviewPoint]) -> Vec<RetentionBucket> {
    let mut buckets: Vec<RetentionBucket> = BUCKETS
        .iter()
        .map(|&(from_days, to_days)| RetentionBucket {
            from_days,
            to_days,
            reviews: 0,
            recalled: 0,
        })
        .collect();
    for p in points.iter().filter(|p| p.elapsed_days >= 1.0) {
        let days = p.elapsed_days.round() as u32;
        if let Some(b) = buckets
            .iter_mut()
            .find(|b| (b.from_days..=b.to_days).contains(&days))
        {
            b.reviews += 1;
            b.recalled += u32::from(p.rating != Rating::Again);
        }
    }
    buckets
}
