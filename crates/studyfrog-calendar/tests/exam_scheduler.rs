#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Deterministic unit tests for ExamPlan validation, propose_prep_weeks, and build/rebalance_exam_schedule.

use std::collections::HashSet;

use studyfrog_calendar::exam_plan::{
    ExamPlan, ExamScheduleBlock, ExamTarget, PrepWindow, prep_window_bounds, validate_exam_plan,
};
use studyfrog_calendar::exam_schedule::{
    ExamSchedItem, build_exam_schedule, rebalance_exam_schedule,
};
use studyfrog_calendar::exam_window::{ExamPrepStats, propose_prep_weeks};
use studyfrog_core::civil::{Date, date_to_days};
use studyfrog_core::uid::Uid;

fn d(year: i32, month: u8, day: u8) -> Date {
    Date { year, month, day }
}

#[test]
fn validate_exam_plan_checks_title_date_weeks_and_targets() {
    let today = d(2026, 11, 26);
    let valid = ExamPlan {
        id: Uid(1),
        title: "Psychologie Klausur".into(),
        exam_date: d(2026, 12, 20),
        window: PrepWindow::UserWeeks(3),
        targets: vec![ExamTarget::Stack("psych".into())],
        blocks: Vec::new(),
        created_ms: 0,
        updated_ms: 0,
    };
    assert!(validate_exam_plan(&valid, today).is_ok());

    let empty_title = ExamPlan {
        title: "   ".into(),
        ..valid.clone()
    };
    assert!(validate_exam_plan(&empty_title, today).is_err());

    let past_date = ExamPlan {
        exam_date: today,
        ..valid.clone()
    };
    assert!(validate_exam_plan(&past_date, today).is_err());

    let zero_weeks = ExamPlan {
        window: PrepWindow::UserWeeks(0),
        ..valid.clone()
    };
    assert!(validate_exam_plan(&zero_weeks, today).is_err());

    let no_targets = ExamPlan {
        targets: Vec::new(),
        ..valid
    };
    assert!(validate_exam_plan(&no_targets, today).is_err());
}

#[test]
fn propose_prep_weeks_is_monotonic_and_clamped_to_exam_date() {
    let today = d(2026, 11, 1);
    let exam = d(2026, 12, 20); // 7 weeks away

    let small_easy = ExamPrepStats {
        total_items: 20,
        new_items: 2,
        avg_familiarity: 0.85,
        avg_stability_days: 25.0,
        recent_accuracy: 0.92,
        lapse_rate: 0.02,
        daily_capacity: 25,
    };
    let large_hard = ExamPrepStats {
        total_items: 180,
        new_items: 140,
        avg_familiarity: 0.25,
        avg_stability_days: 2.0,
        recent_accuracy: 0.55,
        lapse_rate: 0.25,
        daily_capacity: 20,
    };

    let p_easy = propose_prep_weeks(&small_easy, today, exam);
    let p_hard = propose_prep_weeks(&large_hard, today, exam);
    assert!(p_hard.weeks >= p_easy.weeks);
    assert!(p_hard.weeks <= 7);
    assert!(!p_hard.reasons.is_empty());

    // Exam tomorrow clamps to 1 week
    let tomorrow = d(2026, 11, 2);
    let p_tight = propose_prep_weeks(&large_hard, today, tomorrow);
    assert_eq!(p_tight.weeks, 1);
}

#[test]
fn build_and_rebalance_cover_all_items_within_bounds_and_prioritize_weak() {
    let today = d(2026, 11, 1);
    let exam = d(2026, 11, 22); // 3 weeks
    let items: Vec<ExamSchedItem> = (1..=12u128)
        .map(|i| ExamSchedItem {
            uid: Uid(i),
            is_new: i <= 4,
            familiarity: (i as f32) / 12.0,
            stability_days: i as f32,
        })
        .collect();

    let out = build_exam_schedule(&items, today, exam, 3, 4);
    assert!(!out.blocks.is_empty());
    let (start, end) = prep_window_bounds(today, exam, 3).unwrap();
    let mut covered = HashSet::new();
    for b in &out.blocks {
        assert!(date_to_days(b.date) >= date_to_days(start));
        assert!(date_to_days(b.date) <= date_to_days(end));
        for u in &b.item_uids {
            covered.insert(*u);
        }
    }
    assert_eq!(covered.len(), 12, "every linked item must be scheduled");
    // Unstudied item Uid(1) must appear in the very first block
    assert!(out.blocks[0].item_uids.contains(&Uid(1)));

    // Rebalance from Nov 10 with a missed block in the past and an added item Uid(13)
    let later = d(2026, 11, 10);
    let past_block = ExamScheduleBlock {
        date: d(2026, 11, 5),
        item_uids: vec![Uid(1)],
        new_count: 1,
        review_count: 0,
    };
    let mut extended = items;
    extended.push(ExamSchedItem {
        uid: Uid(13),
        is_new: true,
        familiarity: 0.0,
        stability_days: 0.0,
    });
    let rebal = rebalance_exam_schedule(&[past_block], &extended, later, exam, 3, 4);
    assert_eq!(rebal.blocks[0].date, d(2026, 11, 5));
    let all_rebal: HashSet<Uid> = rebal
        .blocks
        .iter()
        .flat_map(|b| b.item_uids.iter().copied())
        .collect();
    assert!(all_rebal.contains(&Uid(13)));
}
