#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The estimated difficulty of each item (EPIC-060): the engine's index averaged per item, hardest
//! first, titled from the index.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;
use studyfrog_report::difficulty::{Observation, empirical_difficulty};
use studyfrog_service::cognitive_items::{MAX_ITEMS, item_difficulties};
use studyfrog_store::telemetry::TelemetryPoint;

fn point(item: u128, latency_ms: u32, semantic: Option<f32>) -> TelemetryPoint {
    TelemetryPoint {
        item: Uid(item),
        reviewed_at_ms: 1_000,
        latency_ms,
        dwell_ms: latency_ms,
        semantic_score: semantic,
        text_complexity: 10,
    }
}

fn titles(items: &[(u128, &str)]) -> HashMap<Uid, (String, String)> {
    items
        .iter()
        .map(|(i, t)| (Uid(*i), ((*t).to_owned(), "Biology".to_owned())))
        .collect()
}

#[test]
fn the_slow_wrong_item_is_harder_and_listed_first() {
    let telemetry = [point(1, 20_000, Some(0.1)), point(2, 1_500, Some(1.0))];
    let rows = item_difficulties(&telemetry, &titles(&[(1, "Krebs cycle"), (2, "Mitosis")]));
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].title, "Krebs cycle");
    assert!(rows[0].difficulty > rows[1].difficulty);
    assert_eq!(rows[0].folder, "Biology");
    assert_eq!(rows[0].answers, 1);
}

#[test]
fn the_difficulty_is_the_mean_of_the_engine_over_the_answers() {
    let telemetry = [point(1, 4_000, None), point(1, 12_000, Some(0.5))];
    let rows = item_difficulties(&telemetry, &titles(&[(1, "Cell")]));
    let expected = telemetry
        .iter()
        .map(|p| {
            empirical_difficulty(&Observation {
                latency_ms: f64::from(p.latency_ms),
                dwell_ms: f64::from(p.dwell_ms),
                words: f64::from(p.text_complexity),
                semantic: p.semantic_score.map(f64::from),
            })
        })
        .sum::<f64>()
        / 2.0;
    assert_eq!(rows[0].answers, 2);
    assert!((rows[0].difficulty - expected).abs() < 1e-9);
}

#[test]
fn an_item_no_longer_indexed_keeps_its_row_with_no_title() {
    let rows = item_difficulties(&[point(7, 3_000, None)], &HashMap::new());
    assert_eq!(rows.len(), 1);
    assert!(rows[0].title.is_empty());
}

#[test]
fn the_list_is_capped() {
    let telemetry: Vec<TelemetryPoint> = (1..=(MAX_ITEMS as u128 + 5))
        .map(|i| point(i, 2_000 + i as u32 * 500, None))
        .collect();
    let rows = item_difficulties(&telemetry, &HashMap::new());
    assert_eq!(rows.len(), MAX_ITEMS);
    assert!(rows.windows(2).all(|w| w[0].difficulty >= w[1].difficulty));
}
