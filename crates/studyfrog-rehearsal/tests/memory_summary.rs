#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for Memory Mode timing, danger levels, and the victory summary.

use studyfrog_rehearsal::memory::{flip_tile, new_memory_session, tick_memory_timer};
use studyfrog_rehearsal::memory_deck::TileRole;
use studyfrog_rehearsal::memory_summary::{
    DangerLevel, MemoryTimer, danger_level, defeat_message, hud_clock_ms, victory_summary,
};

fn cards(n: usize) -> Vec<(String, String)> {
    (0..n).map(|i| (format!("q{i}"), format!("a{i}"))).collect()
}

/// A finished 2-pair session: two turns, both matched, completed at `at_ms`.
fn finished(at_ms: u64, limit: Option<u64>) -> studyfrog_rehearsal::memory::MemorySession {
    let mut s = new_memory_session(&cards(2), 2, 4, 1000, limit);
    s = tick_memory_timer(&s, at_ms);
    for pair in 0..2u32 {
        let f = s
            .tiles
            .iter()
            .find(|t| t.pair == pair && t.role == TileRole::Front)
            .unwrap()
            .index;
        let b = s
            .tiles
            .iter()
            .find(|t| t.pair == pair && t.role == TileRole::Back)
            .unwrap()
            .index;
        s = flip_tile(&s, f).unwrap();
        s = flip_tile(&s, b).unwrap();
    }
    s
}

#[test]
fn danger_rises_only_for_time_attack() {
    assert_eq!(
        danger_level(MemoryTimer::Stopwatch, 999_999),
        DangerLevel::Calm
    );
    assert_eq!(
        danger_level(MemoryTimer::TimeAttack(60), 10_000),
        DangerLevel::Calm
    );
    assert_eq!(
        danger_level(MemoryTimer::TimeAttack(60), 35_000),
        DangerLevel::Warning
    );
    assert_eq!(
        danger_level(MemoryTimer::TimeAttack(60), 51_000),
        DangerLevel::Critical
    );
    assert_eq!(
        danger_level(MemoryTimer::TimeAttack(60), 70_000),
        DangerLevel::Critical
    );
}

#[test]
fn victory_summary_reports_time_moves_accuracy_and_bonus() {
    let s = finished(30_000, Some(60_000));
    let v = victory_summary(&s, MemoryTimer::TimeAttack(60), None).unwrap();
    assert_eq!(v.elapsed_ms, 30_000);
    assert_eq!(v.moves, 2);
    assert_eq!(v.pairs, 2);
    assert!((v.accuracy_pct - 100.0).abs() < 1e-3);
    assert!((v.speed_score - 4.0).abs() < 1e-3);
    assert_eq!(v.bonus_points, 300);
    assert!(v.personal_best);
}

#[test]
fn stopwatch_has_no_bonus_and_personal_best_needs_a_faster_time() {
    let s = finished(45_000, None);
    let slower = victory_summary(&s, MemoryTimer::Stopwatch, Some(40_000)).unwrap();
    assert_eq!(slower.bonus_points, 0);
    assert!(!slower.personal_best);
    let faster = victory_summary(&s, MemoryTimer::Stopwatch, Some(50_000)).unwrap();
    assert!(faster.personal_best);
}

#[test]
fn no_summary_while_playing_or_after_a_timeout() {
    let playing = new_memory_session(&cards(2), 2, 4, 1000, None);
    assert!(victory_summary(&playing, MemoryTimer::Stopwatch, None).is_none());
    let timed_out = tick_memory_timer(
        &new_memory_session(&cards(2), 2, 4, 1000, Some(5_000)),
        5_000,
    );
    assert!(victory_summary(&timed_out, MemoryTimer::TimeAttack(5), None).is_none());
    assert!(!defeat_message().is_empty());
}

#[test]
fn the_hud_clock_counts_down_in_a_time_attack_and_stops_at_zero() {
    assert_eq!(hud_clock_ms(MemoryTimer::TimeAttack(60), 0), 60_000);
    assert_eq!(hud_clock_ms(MemoryTimer::TimeAttack(60), 7_000), 53_000);
    assert_eq!(hud_clock_ms(MemoryTimer::TimeAttack(60), 64_000), 0);
    assert_eq!(hud_clock_ms(MemoryTimer::Stopwatch, 64_000), 64_000);
}
