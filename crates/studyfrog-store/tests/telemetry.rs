#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for telemetry persistence: round trip, item and range filters, and validation.

use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::telemetry::{
    TelemetryPoint, load_telemetry_for_items, load_telemetry_range, record_telemetry,
};

fn point(item: u128, at: i64, semantic: Option<f32>) -> TelemetryPoint {
    TelemetryPoint {
        item: Uid(item),
        reviewed_at_ms: at,
        latency_ms: 4_200,
        dwell_ms: 6_000,
        semantic_score: semantic,
        text_complexity: 37,
    }
}

#[test]
fn records_load_by_item_and_by_range_oldest_first() {
    let mut conn = open_memory().unwrap();
    studyfrog_store::migrate::migrate(&mut conn, MIGRATIONS).unwrap();
    record_telemetry(&conn, &point(1, 2_000, Some(0.8))).unwrap();
    record_telemetry(&conn, &point(2, 1_000, None)).unwrap();
    record_telemetry(&conn, &point(1, 3_000, Some(0.4))).unwrap();
    record_telemetry(&conn, &point(1, 9_000, Some(0.9))).unwrap();

    let mine = load_telemetry_for_items(&conn, &[Uid(1)], 0, 5_000).unwrap();
    assert_eq!(mine.len(), 2);
    assert_eq!(mine[0].reviewed_at_ms, 2_000);
    assert!((mine[0].semantic_score.unwrap() - 0.8).abs() < 1e-6);

    let both = load_telemetry_range(&conn, 0, 10_000).unwrap();
    let times: Vec<i64> = both.iter().map(|p| p.reviewed_at_ms).collect();
    assert_eq!(times, vec![1_000, 2_000, 3_000, 9_000]);
    assert!(both[0].semantic_score.is_none());
}

#[test]
fn a_semantic_score_outside_the_unit_interval_is_refused() {
    let mut conn = open_memory().unwrap();
    studyfrog_store::migrate::migrate(&mut conn, MIGRATIONS).unwrap();
    assert!(record_telemetry(&conn, &point(1, 1, Some(1.5))).is_err());
    assert!(load_telemetry_range(&conn, 0, 10).unwrap().is_empty());
}
