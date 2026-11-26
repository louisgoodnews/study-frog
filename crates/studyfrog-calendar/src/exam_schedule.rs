//! Pure windowed item distribution and re-balancing scheduler for exams (EPIC-076, TASK-0308).
//!
//! Complies with Invariant P1: zero inherent impl blocks; pure free functions with injected `today`.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};

use studyfrog_core::civil::{Date, add_days, date_to_days};
use studyfrog_core::uid::Uid;

use crate::exam_plan::{ExamScheduleBlock, prep_window_bounds};

/// Lightweight item metadata needed to prioritize and schedule exam items.
#[derive(Debug, Clone, PartialEq)]
pub struct ExamSchedItem {
    /// Item UID.
    pub uid: Uid,
    /// True when the item has never been reviewed.
    pub is_new: bool,
    /// Current familiarity (`0.0..=1.0`, `0.0` if unstudied).
    pub familiarity: f32,
    /// Current FSRS stability in days (`0.0` if unstudied).
    pub stability_days: f32,
}

/// Output of scheduling or re-balancing an exam's items across a preparation window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExamScheduleOutput {
    /// Daily study blocks ordered chronologically.
    pub blocks: Vec<ExamScheduleBlock>,
    /// Non-fatal capacity or timeline warnings.
    pub warnings: Vec<String>,
}

fn priority_cmp(a: &ExamSchedItem, b: &ExamSchedItem) -> Ordering {
    // Unstudied and lowest-familiarity items go earliest so they get multiple spaced passes.
    b.is_new
        .cmp(&a.is_new)
        .then_with(|| a.familiarity.total_cmp(&b.familiarity))
        .then_with(|| a.stability_days.total_cmp(&b.stability_days))
        .then_with(|| a.uid.cmp(&b.uid))
}

fn push_unique(day_items: &mut Vec<Uid>, uid: Uid) {
    if !day_items.contains(&uid) {
        day_items.push(uid);
    }
}

/// Distribute `items` across `[max(today, exam_date - weeks * 7), exam_date - 1]`.
pub fn build_exam_schedule(
    items: &[ExamSchedItem],
    today: Date,
    exam_date: Date,
    weeks: u16,
    daily_capacity: usize,
) -> ExamScheduleOutput {
    let Some((start, end)) = prep_window_bounds(today, exam_date, weeks) else {
        return ExamScheduleOutput {
            blocks: Vec::new(),
            warnings: vec!["No available days before exam date.".to_owned()],
        };
    };
    if items.is_empty() {
        return ExamScheduleOutput::default();
    }

    let total_days = (date_to_days(end) - date_to_days(start) + 1).max(1) as usize;
    let cap = daily_capacity.max(1);

    let mut sorted = items.to_vec();
    sorted.sort_by(priority_cmp);

    let mut by_day: BTreeMap<i64, (Vec<Uid>, usize, usize)> = BTreeMap::new();
    let start_days = date_to_days(start);
    for d in 0..total_days {
        by_day.insert(start_days + d as i64, (Vec::new(), 0, 0));
    }

    // Reserve the final 1 day (when window >= 3 days) for consolidation passes; introduce items in early days.
    let intro_days = if total_days >= 3 {
        (total_days - 1).max(1)
    } else {
        total_days
    };

    let mut overflow = false;
    for (idx, item) in sorted.iter().enumerate() {
        let day_offset = (idx / cap).min(intro_days - 1);
        let d1 = start_days + day_offset as i64;
        if let Some((uids, new_c, rev_c)) = by_day.get_mut(&d1) {
            if uids.len() >= cap {
                overflow = true;
            }
            push_unique(uids, item.uid);
            if item.is_new {
                *new_c += 1;
            } else {
                *rev_c += 1;
            }
        }

        // Schedule a spaced follow-up consolidation pass for new or weak items (< 0.65 familiarity).
        if (item.is_new || item.familiarity < 0.65) && total_days >= 3 {
            let gap = ((total_days - 1 - day_offset) / 2).clamp(1, 5);
            let d2 = (d1 + gap as i64).min(date_to_days(end));
            if d2 != d1
                && let Some((uids, _, rev_c)) = by_day.get_mut(&d2)
            {
                push_unique(uids, item.uid);
                *rev_c += 1;
            }
        }
    }

    let mut warnings = Vec::new();
    if overflow {
        warnings.push(format!(
            "{} items exceed {} items/day over {} day(s); extra items were packed into the window.",
            sorted.len(),
            cap,
            intro_days
        ));
    }

    let blocks = by_day
        .into_iter()
        .filter(|(_, (uids, _, _))| !uids.is_empty())
        .map(
            |(day, (item_uids, new_count, review_count))| ExamScheduleBlock {
                date: add_days(start, day - start_days),
                item_uids,
                new_count,
                review_count,
            },
        )
        .collect();

    ExamScheduleOutput { blocks, warnings }
}

/// Re-balance remaining exam blocks from `today` forward, preserving completed past blocks and carrying missed items.
pub fn rebalance_exam_schedule(
    existing: &[ExamScheduleBlock],
    items: &[ExamSchedItem],
    today: Date,
    exam_date: Date,
    weeks: u16,
    daily_capacity: usize,
) -> ExamScheduleOutput {
    let today_days = date_to_days(today);
    let item_set: HashSet<Uid> = items.iter().map(|i| i.uid).collect();
    let kept_past: Vec<ExamScheduleBlock> = existing
        .iter()
        .filter(|b| date_to_days(b.date) < today_days)
        .cloned()
        .map(|mut b| {
            b.item_uids.retain(|u| item_set.contains(u));
            b
        })
        .filter(|b| !b.item_uids.is_empty())
        .collect();

    let mut future = build_exam_schedule(items, today, exam_date, weeks, daily_capacity);
    let mut combined = kept_past;
    combined.append(&mut future.blocks);
    ExamScheduleOutput {
        blocks: combined,
        warnings: future.warnings,
    }
}
