#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Unit tests for priority-based queue ordering and session budget slicing.

use studyfrog_core::memory::MemState;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::priority::Priority;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::candidate::Candidate;
use studyfrog_service::priority_queue::{
    build_priority_queue, candidate_familiarity, candidate_retrievability, is_due_or_new,
    priority_to_f64,
};
use studyfrog_srs::params::params_default;
use studyfrog_srs::priority_slice::PriorityWeights;

const NOW: UnixMs = UnixMs(1_700_000_000_000);
const HOUR_MS: i64 = 3_600_000;
const DAY_MS: i64 = 86_400_000;

fn make_candidate(
    uid: u128,
    priority: Priority,
    memory: Option<MemState>,
    due: Option<UnixMs>,
    last_review: Option<UnixMs>,
    familiarity: Option<f32>,
) -> Candidate<'static> {
    Candidate {
        uid: Uid(uid),
        kind: ItemKind::Card,
        stack: Uid(1),
        also_in: &[],
        tags: &[],
        position: uid as u32,
        memory,
        due,
        last_review,
        priority,
        difficulty_hint: None,
        familiarity,
    }
}

#[test]
fn priority_to_f64_maps_all_levels() {
    assert_eq!(priority_to_f64(Priority::Lowest), 0.0);
    assert_eq!(priority_to_f64(Priority::Low), 0.25);
    assert_eq!(priority_to_f64(Priority::Medium), 0.50);
    assert_eq!(priority_to_f64(Priority::High), 0.75);
    assert_eq!(priority_to_f64(Priority::Highest), 1.0);
}

#[test]
fn retrievability_handles_mem_new_and_overdue() {
    let p = params_default();

    // 1. Unreviewed item defaults to 0.9
    let unreviewed = make_candidate(1, Priority::Medium, None, None, None, None);
    assert!((candidate_retrievability(&unreviewed, NOW, &p) - 0.9).abs() < 1e-6);

    // 2. Item with memory reviewed right now has retrievability 1.0
    let fresh_mem = MemState {
        stability: 10.0,
        difficulty: 5.0,
    };
    let fresh = make_candidate(2, Priority::Medium, Some(fresh_mem), None, Some(NOW), None);
    let r_fresh = candidate_retrievability(&fresh, NOW, &p);
    assert!((r_fresh - 1.0).abs() < 1e-5);

    // 3. Item with memory reviewed 10 days ago (stability = 10) has retrievability ~0.9
    let ten_days_ago = UnixMs(NOW.0 - 10 * DAY_MS);
    let due_item = make_candidate(
        3,
        Priority::Medium,
        Some(fresh_mem),
        None,
        Some(ten_days_ago),
        None,
    );
    let r_due = candidate_retrievability(&due_item, NOW, &p);
    assert!((r_due - 0.9).abs() < 0.01);

    // 4. Item reviewed without memory: retrievability drops as overdue time grows
    let non_mem_due_now = make_candidate(4, Priority::Medium, None, Some(NOW), None, None);
    let non_mem_overdue = make_candidate(
        5,
        Priority::Medium,
        None,
        Some(UnixMs(NOW.0 - 5 * DAY_MS)),
        None,
        None,
    );
    let r_due_now = candidate_retrievability(&non_mem_due_now, NOW, &p);
    let r_overdue = candidate_retrievability(&non_mem_overdue, NOW, &p);
    assert!((r_due_now - 0.9).abs() < 1e-6);
    assert!(r_overdue < r_due_now);
}

#[test]
fn familiarity_defaults_to_zero_when_none() {
    let no_fam = make_candidate(1, Priority::Medium, None, None, None, None);
    assert_eq!(candidate_familiarity(&no_fam), 0.0);

    let with_fam = make_candidate(2, Priority::Medium, None, None, None, Some(0.72));
    assert!((candidate_familiarity(&with_fam) - 0.72).abs() < 1e-5);
}

#[test]
fn is_due_or_new_filters_properly() {
    let new_item = make_candidate(1, Priority::Medium, None, None, None, None);
    assert!(is_due_or_new(&new_item, NOW));

    let due_item = make_candidate(
        2,
        Priority::Medium,
        None,
        Some(UnixMs(NOW.0 - HOUR_MS)),
        Some(UnixMs(NOW.0 - 24 * HOUR_MS)),
        None,
    );
    assert!(is_due_or_new(&due_item, NOW));

    let not_due_item = make_candidate(
        3,
        Priority::Medium,
        None,
        Some(UnixMs(NOW.0 + HOUR_MS)),
        Some(UnixMs(NOW.0 - 12 * HOUR_MS)),
        None,
    );
    assert!(!is_due_or_new(&not_due_item, NOW));
}

#[test]
fn build_priority_queue_empty_and_zero_budget() {
    let empty: [Candidate; 0] = [];
    assert_eq!(
        build_priority_queue(&empty, 10, None, NOW),
        Vec::<Uid>::new()
    );

    let item = make_candidate(1, Priority::Highest, None, None, None, None);
    assert_eq!(
        build_priority_queue(&[item], 0, None, NOW),
        Vec::<Uid>::new()
    );
}

#[test]
fn build_priority_queue_filters_non_due_and_slices_budget() {
    let items = [
        make_candidate(1, Priority::Lowest, None, None, None, None),
        make_candidate(
            2,
            Priority::Highest,
            None,
            Some(UnixMs(NOW.0 + 24 * HOUR_MS)),
            Some(UnixMs(NOW.0 - HOUR_MS)),
            None,
        ), // future due: excluded
        make_candidate(3, Priority::Highest, None, None, None, None),
        make_candidate(4, Priority::High, None, None, None, None),
    ];

    // Budget 2 should exclude item 2 (not due), and pick the top 2 from items 3, 4, 1.
    // Item 3 is Highest, Item 4 is High, Item 1 is Lowest.
    let queue = build_priority_queue(&items, 2, None, NOW);
    assert_eq!(queue, vec![Uid(3), Uid(4)]);
}

#[test]
fn build_priority_queue_respects_custom_weights() {
    // Two new items: item 1 has High priority and low familiarity, item 2 has Lowest priority but familiarity 0.
    // If weights favor retrievability/familiarity shortfall vs priority:
    let item_high_p = make_candidate(1, Priority::Highest, None, None, None, Some(1.0));
    let item_low_p = make_candidate(2, Priority::Lowest, None, None, None, Some(0.0));

    // Priority-heavy weights: item 1 wins
    let p_weights = PriorityWeights {
        priority_weight: 1.0,
        retrievability_weight: 0.0,
        familiarity_weight: 0.0,
    };
    let q_p = build_priority_queue(&[item_high_p, item_low_p], 2, Some(&p_weights), NOW);
    assert_eq!(q_p, vec![Uid(1), Uid(2)]);

    // Familiarity-heavy weights: item 2 (familiarity shortfall 1.0) beats item 1 (shortfall 0.0)
    let f_weights = PriorityWeights {
        priority_weight: 0.0,
        retrievability_weight: 0.0,
        familiarity_weight: 1.0,
    };
    let q_f = build_priority_queue(&[item_high_p, item_low_p], 2, Some(&f_weights), NOW);
    assert_eq!(q_f, vec![Uid(2), Uid(1)]);
}
