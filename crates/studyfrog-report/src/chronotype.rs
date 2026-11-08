//! When the learner remembers best: recall, volume and answer time by local hour of the day and by
//! weekday, with the peak and trough windows (STORY-0102). Pure.
//!
//! Every review and every answer time carries its own UTC offset, so a history that spans a clock
//! change or a journey is placed in the local hour it happened in.

use studyfrog_core::review::Rating;

use crate::latency::percentile;

/// Reviews a bucket needs by default before it can be part of a peak or trough.
pub const MIN_BUCKET_REVIEWS: u32 = 20;
/// Hours in a peak or trough window.
pub const WINDOW_HOURS: usize = 2;

const MS_PER_HOUR: i64 = 3_600_000;
const MS_PER_DAY: i64 = 86_400_000;

/// A review at a moment, with the local UTC offset then.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedReview {
    /// Unix milliseconds.
    pub at_ms: i64,
    /// Local UTC offset in minutes (east positive).
    pub offset_minutes: i32,
    /// The rating.
    pub rating: Rating,
}

/// An answer time at a moment, with the local UTC offset then.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedLatency {
    /// Unix milliseconds.
    pub at_ms: i64,
    /// Local UTC offset in minutes.
    pub offset_minutes: i32,
    /// Time to reveal or answer, paused time excluded.
    pub ms: u32,
}

/// The numbers of one hour or weekday.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClockBucket {
    /// Reviews.
    pub reviews: u32,
    /// Not rated Again.
    pub recalled: u32,
    /// Rated Good or Easy.
    pub good_easy: u32,
    /// Answer times recorded.
    pub answers: u32,
    /// Median answer time.
    pub median_ms: Option<u32>,
    /// 90th percentile of the answer time.
    pub p90_ms: Option<u32>,
}

/// A stretch of consecutive hours (it may wrap past midnight).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockWindow {
    /// First local hour.
    pub start_hour: u8,
    /// Hours covered.
    pub hours: u8,
    /// Recall share in the window.
    pub retention: f32,
    /// Median answer time in the window.
    pub median_ms: Option<u32>,
    /// Reviews in the window.
    pub reviews: u32,
}

/// The whole profile.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChronotypeReport {
    /// 24 local hours, 0 first.
    pub hours: Vec<ClockBucket>,
    /// 7 weekdays, Monday first.
    pub weekdays: Vec<ClockBucket>,
    /// The window where recall is highest for its answer time.
    pub peak: Option<ClockWindow>,
    /// The window where it is lowest (not overlapping the peak).
    pub trough: Option<ClockWindow>,
    /// The weekday with the best recall (index, Monday 0), among those with enough reviews.
    pub best_weekday: Option<usize>,
    /// The weekday with the worst recall.
    pub worst_weekday: Option<usize>,
    /// The minimum reviews per bucket that was used.
    pub min_reviews: u32,
    /// Median of every answer time.
    pub overall_median_ms: Option<u32>,
}

fn local_ms(at_ms: i64, offset_minutes: i32) -> i64 {
    at_ms + i64::from(offset_minutes) * 60_000
}

/// Local hour 0..24 of a moment.
pub fn local_hour(at_ms: i64, offset_minutes: i32) -> usize {
    (local_ms(at_ms, offset_minutes).rem_euclid(MS_PER_DAY) / MS_PER_HOUR) as usize
}

/// Local weekday of a moment, Monday 0 (1970-01-01 was a Thursday).
pub fn local_weekday(at_ms: i64, offset_minutes: i32) -> usize {
    (local_ms(at_ms, offset_minutes).div_euclid(MS_PER_DAY) + 3).rem_euclid(7) as usize
}

fn add_review(b: &mut ClockBucket, rating: Rating) {
    b.reviews += 1;
    b.recalled += u32::from(rating != Rating::Again);
    b.good_easy += u32::from(matches!(rating, Rating::Good | Rating::Easy));
}

fn finish(b: &mut ClockBucket, ms: &mut [u32]) {
    ms.sort_unstable();
    b.answers = ms.len() as u32;
    (b.median_ms, b.p90_ms) = (percentile(ms, 50), percentile(ms, 90));
}

fn rate(reviews: u32, recalled: u32) -> f32 {
    recalled as f32 / reviews.max(1) as f32
}

/// Recall for the time it takes: `retention / ln(median_ms)` (higher is better).
fn score(w: &ClockWindow, fallback_ms: u32) -> f32 {
    let ms = w.median_ms.unwrap_or(fallback_ms).max(2) as f32;
    w.retention / ms.ln()
}

/// The median answer time of a window, approximated by the mean of its hourly medians weighted by
/// their answers.
fn window_median(hours: &[&ClockBucket]) -> Option<u32> {
    let (sum, n) = hours.iter().fold((0_u64, 0_u64), |(s, n), b| {
        let k = u64::from(b.answers);
        (s + u64::from(b.median_ms.unwrap_or(0)) * k, n + k)
    });
    (n > 0).then(|| (sum / n) as u32)
}

fn window(hours: &[ClockBucket], start: usize, min: u32) -> Option<ClockWindow> {
    let parts: Vec<&ClockBucket> = (0..WINDOW_HOURS)
        .map(|k| &hours[(start + k) % 24])
        .collect();
    if parts.iter().any(|b| b.reviews < min) {
        return None;
    }
    let (reviews, recalled) = parts
        .iter()
        .fold((0, 0), |(n, r), b| (n + b.reviews, r + b.recalled));
    Some(ClockWindow {
        start_hour: start as u8,
        hours: WINDOW_HOURS as u8,
        retention: rate(reviews, recalled),
        median_ms: window_median(&parts),
        reviews,
    })
}

fn overlaps(a: &ClockWindow, b: &ClockWindow) -> bool {
    let d = (i32::from(a.start_hour) - i32::from(b.start_hour)).rem_euclid(24);
    d < i32::from(a.hours) || 24 - d < i32::from(b.hours)
}

fn extreme_weekday(days: &[ClockBucket], min: u32, best: bool) -> Option<usize> {
    let rated = days.iter().enumerate().filter(|(_, d)| d.reviews >= min);
    let key = |d: &ClockBucket| rate(d.reviews, d.recalled);
    if best {
        rated
            .max_by(|a, b| key(a.1).total_cmp(&key(b.1)))
            .map(|x| x.0)
    } else {
        rated
            .min_by(|a, b| key(a.1).total_cmp(&key(b.1)))
            .map(|x| x.0)
    }
}

/// The profile of `reviews` and `latencies`; buckets with fewer than `min_reviews` reviews are not
/// used for the peak, trough, best and worst day.
pub fn chronotype(
    reviews: &[TimedReview],
    latencies: &[TimedLatency],
    min_reviews: u32,
) -> ChronotypeReport {
    let (mut hours, mut days) = (
        vec![ClockBucket::default(); 24],
        vec![ClockBucket::default(); 7],
    );
    let (mut hour_ms, mut day_ms) = (vec![Vec::new(); 24], vec![Vec::new(); 7]);
    for r in reviews {
        add_review(&mut hours[local_hour(r.at_ms, r.offset_minutes)], r.rating);
        add_review(
            &mut days[local_weekday(r.at_ms, r.offset_minutes)],
            r.rating,
        );
    }
    for l in latencies {
        hour_ms[local_hour(l.at_ms, l.offset_minutes)].push(l.ms);
        day_ms[local_weekday(l.at_ms, l.offset_minutes)].push(l.ms);
    }
    hours
        .iter_mut()
        .zip(hour_ms.iter_mut())
        .for_each(|(b, m)| finish(b, m));
    days.iter_mut()
        .zip(day_ms.iter_mut())
        .for_each(|(b, m)| finish(b, m));
    let mut all: Vec<u32> = latencies.iter().map(|l| l.ms).collect();
    all.sort_unstable();
    let base = ChronotypeReport {
        hours,
        weekdays: days,
        overall_median_ms: percentile(&all, 50),
        ..ChronotypeReport::default()
    };
    with_windows(base, min_reviews)
}

/// The report with its peak, trough, best and worst weekday chosen anew for `min_reviews` (the
/// buckets stay as they are).
pub fn with_windows(r: ChronotypeReport, min_reviews: u32) -> ChronotypeReport {
    let fallback = r.overall_median_ms.unwrap_or(3_000);
    let windows: Vec<ClockWindow> = (0..24)
        .filter_map(|s| window(&r.hours, s, min_reviews))
        .collect();
    let by_score =
        |a: &&ClockWindow, b: &&ClockWindow| score(a, fallback).total_cmp(&score(b, fallback));
    let peak = windows.iter().max_by(by_score).copied();
    let trough = windows
        .iter()
        .filter(|w| peak.is_none_or(|p| !overlaps(&p, w)))
        .min_by(by_score)
        .copied();
    ChronotypeReport {
        best_weekday: extreme_weekday(&r.weekdays, min_reviews, true),
        worst_weekday: extreme_weekday(&r.weekdays, min_reviews, false),
        peak,
        trough,
        min_reviews,
        ..r
    }
}
