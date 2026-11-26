//! Adaptive preparation-window estimator from vault retention stats (EPIC-076, TASK-0307).
//!
//! Complies with Invariant P1: zero inherent impl blocks; pure free functions with injected `today`.

use studyfrog_core::civil::{Date, date_to_days};

/// Aggregated stats of the items linked to an exam, used to propose a preparation window.
#[derive(Debug, Clone, PartialEq)]
pub struct ExamPrepStats {
    /// Total number of linked items (cards, questions, notes).
    pub total_items: usize,
    /// Number of unstudied / never-reviewed items.
    pub new_items: usize,
    /// Mean familiarity on `0.0..=1.0` across studied items (`0.0` when none studied).
    pub avg_familiarity: f32,
    /// Mean FSRS stability in days across items with memory state (`0.0` when none).
    pub avg_stability_days: f32,
    /// Recent review accuracy on `0.0..=1.0` (`1.0` when no history).
    pub recent_accuracy: f32,
    /// Share of items flagged as leeches or repeatedly lapsed (`0.0..=1.0`).
    pub lapse_rate: f32,
    /// Target daily review + learning capacity in items/day.
    pub daily_capacity: usize,
}

/// Result of proposing a preparation window in weeks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedWindow {
    /// Recommended preparation window in weeks (clamped to `[1, max_available_weeks]`).
    pub weeks: u16,
    /// Human-readable factor explanations.
    pub reasons: Vec<String>,
}

/// Pure free function proposing a preparation window in weeks from item and learner stats.
pub fn propose_prep_weeks(stats: &ExamPrepStats, today: Date, exam_date: Date) -> ProposedWindow {
    let days_left = (date_to_days(exam_date) - date_to_days(today)).max(1);
    let max_weeks = ((days_left + 6) / 7).max(1) as u16;
    if stats.total_items == 0 {
        return ProposedWindow {
            weeks: 1,
            reasons: vec!["No items linked yet; defaulting to 1 week.".to_owned()],
        };
    }

    let cap = stats.daily_capacity.max(5) as f64;
    let total = stats.total_items as f64;
    let new_share = (stats.new_items as f64 / total).clamp(0.0, 1.0);
    let fam = f64::from(stats.avg_familiarity.clamp(0.0, 1.0));
    let acc = f64::from(stats.recent_accuracy.clamp(0.2, 1.0));
    let lapse = f64::from(stats.lapse_rate.clamp(0.0, 1.0));

    // Estimate effective touches per item: new items need ~3 spaced passes; familiar items need ~1-2.
    let passes_per_item = 1.2 + 1.8 * new_share + 1.2 * (1.0 - fam) + 0.8 * lapse;
    let accuracy_factor = (1.0 / acc).clamp(1.0, 1.8);
    let stability_bonus = if stats.avg_stability_days >= 21.0 {
        0.85
    } else if stats.avg_stability_days >= 7.0 {
        1.0
    } else {
        1.15
    };

    let required_touches = total * passes_per_item * accuracy_factor * stability_bonus;
    let needed_days = (required_touches / cap).ceil();
    // Add 2 buffer/consolidation days and convert to weeks (at least 1).
    let raw_weeks = ((needed_days + 2.0) / 7.0).ceil().max(1.0) as u16;
    let weeks = raw_weeks.min(max_weeks).max(1);

    let mut reasons = vec![
        format!(
            "{} linked items ({} unstudied, ~{} items/day pace)",
            stats.total_items,
            stats.new_items,
            stats.daily_capacity.max(5)
        ),
        format!(
            "Avg familiarity {:.0}%, recent accuracy {:.0}%",
            fam * 100.0,
            acc * 100.0
        ),
    ];
    if lapse > 0.15 {
        reasons.push(format!(
            "Higher lapse rate ({:.0}%) adds extra consolidation time",
            lapse * 100.0
        ));
    }
    if raw_weeks > max_weeks {
        reasons.push(format!(
            "Clamped to {max_weeks} week(s) remaining before exam date"
        ));
    }
    ProposedWindow { weeks, reasons }
}
