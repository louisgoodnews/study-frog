#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A timed rating is recorded as telemetry once; an untimed one records nothing (STORY-0176 wiring).

mod fam_support;

use fam_support::{ITEM, Opts, chain, on, vault};
use studyfrog_core::review::Rating::Good;
use studyfrog_service::session::open_vault;
use studyfrog_store::telemetry::load_telemetry_range;

#[test]
fn a_timed_rating_is_recorded_once_with_its_timing() {
    let dir = vault("telemetry-timed", None);
    let opts = Opts {
        timing: Some((3_000, 4_500)),
        ..Default::default()
    };
    chain(&dir, &[Good], &on(), &opts);
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let rows = load_telemetry_range(&session.conn, 0, i64::MAX).unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].item, ITEM);
    // fam_support records latency as half the answer time, dwell as the answer time and 12 words.
    assert_eq!(rows[0].latency_ms, 1_500);
    assert_eq!(rows[0].dwell_ms, 3_000);
    assert_eq!(rows[0].text_complexity, 12);
    assert!(rows[0].semantic_score.is_none());
}

#[test]
fn an_untimed_rating_records_no_telemetry() {
    let dir = vault("telemetry-untimed", None);
    chain(&dir, &[Good], &on(), &Opts::default());
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let rows = load_telemetry_range(&session.conn, 0, i64::MAX).unwrap();
    assert!(rows.is_empty(), "{rows:?}");
}
