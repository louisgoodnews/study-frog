//! Performance of a set of items over a stretch of time: how many reviews, how they went and how long
//! the answers took. Pure; the service picks the reviews that belong to a vault, a stack or an item.

use studyfrog_core::review::{Rating, rating_value};
use studyfrog_core::uid::Uid;

use crate::latency::{FlipSample, percentile};

/// One review as the explorer sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewPoint {
    /// The item.
    pub item: Uid,
    /// The rating given.
    pub rating: Rating,
    /// When, in Unix milliseconds.
    pub at_ms: i64,
}

/// How a set of reviews went.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Perf {
    /// Reviews.
    pub reviews: u32,
    /// Different items reviewed.
    pub items: u32,
    /// Reviews per rating: Again, Hard, Good, Easy.
    pub ratings: [u32; 4],
    /// Median time to reveal an answer, in milliseconds.
    pub median_ms: Option<u32>,
    /// Total time spent revealing answers, in milliseconds.
    pub total_ms: u64,
    /// The latest review.
    pub last_ms: Option<i64>,
}

/// The share of reviews rated Good or Easy, in percent; `None` without reviews.
pub fn success_percent(p: &Perf) -> Option<u32> {
    (p.reviews > 0).then(|| (p.ratings[2] + p.ratings[3]) * 100 / p.reviews)
}

/// The performance of `reviews`, with the answer times of `samples` that belong to the same items.
pub fn perf_of(reviews: &[ReviewPoint], samples: &[FlipSample]) -> Perf {
    let mut ratings = [0_u32; 4];
    let mut seen = std::collections::HashSet::new();
    for r in reviews {
        ratings[usize::from(rating_value(r.rating)) - 1] += 1;
        seen.insert(r.item);
    }
    let mut times: Vec<u32> = samples
        .iter()
        .filter(|s| seen.contains(&s.item))
        .map(|s| s.ms)
        .collect();
    times.sort_unstable();
    Perf {
        reviews: reviews.len() as u32,
        items: seen.len() as u32,
        ratings,
        median_ms: percentile(&times, 50),
        total_ms: times.iter().map(|t| u64::from(*t)).sum(),
        last_ms: reviews.iter().map(|r| r.at_ms).max(),
    }
}

/// Reviews of one day and how many of them were Good or Easy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayPoint {
    /// Days since the Unix epoch, on the user's clock.
    pub day: i64,
    /// Reviews.
    pub reviews: u32,
    /// Of those, rated Good or Easy.
    pub good: u32,
}

const MS_PER_DAY: i64 = 86_400_000;

/// The last `days` days ending at `today` (inclusive), oldest first, with zeros for quiet days.
pub fn daily(
    reviews: &[ReviewPoint],
    today: i64,
    days: usize,
    offset_minutes: i32,
) -> Vec<DayPoint> {
    let first = today - days as i64 + 1;
    let mut out: Vec<DayPoint> = (first..=today)
        .map(|day| DayPoint {
            day,
            reviews: 0,
            good: 0,
        })
        .collect();
    for r in reviews {
        let day = (r.at_ms + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY);
        if let Some(p) = out.get_mut(usize::try_from(day - first).unwrap_or(usize::MAX)) {
            p.reviews += 1;
            p.good += u32::from(matches!(r.rating, Rating::Good | Rating::Easy));
        }
    }
    out
}
