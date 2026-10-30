//! How many reviews are coming. Pure.
//!
//! Each scheduled item is projected forward: it is reviewed on its due day, then again after an
//! interval that grows by a typical factor; every review has a chance of a lapse that adds a review
//! the next day; and new items keep arriving at the recent pace. The result is an expected number of
//! reviews per day, not a promise.

/// An item with a review scheduled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScheduledItem {
    /// The day the review is due (days since 1970-01-01, local).
    pub due_day: i64,
    /// The interval the item has now, in days.
    pub interval_days: f32,
}

/// What the user's history says about how reviews unfold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Model {
    /// Typical factor by which an interval grows from one review to the next.
    pub growth: f32,
    /// Days until the second review of a new item.
    pub first_interval_days: f32,
    /// Share of reviews rated Again; each adds a review the next day.
    pub lapse_rate: f32,
    /// New items started per day.
    pub new_per_day: f32,
}

/// Used when there is too little history to estimate from.
pub fn model_default() -> Model {
    Model {
        growth: 2.0,
        first_interval_days: 2.0,
        lapse_rate: 0.1,
        new_per_day: 0.0,
    }
}

fn add(load: &mut [f32], today: i64, day: i64, weight: f32, lapse: f32) {
    for (d, w) in [(day, weight), (day + 1, weight * lapse)] {
        if let Some(slot) = usize::try_from(d - today)
            .ok()
            .and_then(|i| load.get_mut(i))
        {
            *slot += w;
        }
    }
}

/// Add the reviews of one item (or of `weight` items) starting at `first_day`; the review after that
/// comes `gap` days later and every later gap is `growth` times the one before.
fn project(load: &mut [f32], today: i64, first_day: i64, gap: f32, weight: f32, m: &Model) {
    let (mut day, mut gap) = (first_day.max(today), gap.max(1.0));
    while day - today < load.len() as i64 {
        add(load, today, day, weight, m.lapse_rate);
        day += gap.round() as i64;
        gap = (gap * m.growth.max(1.0)).max(1.0);
    }
}

/// Expected reviews on each of the `days` days starting at `today`. Overdue items count today.
pub fn forecast(items: &[ScheduledItem], m: &Model, today: i64, days: usize) -> Vec<f32> {
    let mut load = vec![0.0; days];
    for it in items {
        project(
            &mut load,
            today,
            it.due_day,
            it.interval_days * m.growth.max(1.0),
            1.0,
            m,
        );
    }
    if m.new_per_day > 0.0 {
        for t in 0..days as i64 {
            project(
                &mut load,
                today,
                today + t,
                m.first_interval_days,
                m.new_per_day,
                m,
            );
        }
    }
    load
}
