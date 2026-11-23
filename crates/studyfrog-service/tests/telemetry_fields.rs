#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Each telemetry field has its own source: the semantic score of a judged answer, the dwell apart
//! from the latency, and the item's words as its complexity (BUG-0031, BUG-0048).

mod fam_support;

use fam_support::{ITEM, on, vault};
use studyfrog_core::event::{EventKind, RunEvent};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunState;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::rehearse::{RatingTiming, StepContext, record_step};
use studyfrog_service::session::open_vault;
use studyfrog_store::telemetry::load_telemetry_range;

const NOW: i64 = 1_800_000_000_000;

/// Rate ITEM once with the given timing, as a run step would.
fn rate_with(
    dir: &fam_support::Dir,
    timing: RatingTiming,
) -> Vec<studyfrog_store::telemetry::TelemetryPoint> {
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let run = Uid(9);
    let out = StepOutput {
        events: vec![RunEvent {
            run,
            seq: 0,
            t_ms: 0,
            item: Some(ITEM),
            kind: EventKind::ItemRated(Rating::Good),
        }],
        latency: None,
        dwell_ms: None,
    };
    let ctx = StepContext {
        run,
        effect: ScheduleEffect::UpdateSchedule,
        state: RunState::Running,
        timing: Some(timing),
    };
    record_step(&mut session.conn, &ctx, &out, UnixMs(NOW), &on()).unwrap();
    load_telemetry_range(&session.conn, 0, i64::MAX).unwrap()
}

fn timing(semantic: Option<f32>) -> RatingTiming {
    RatingTiming {
        answer_ms: 2_000,
        expected_ms: 3_000,
        latency_ms: 1_200,
        dwell_ms: 4_800,
        text_words: 17,
        semantic,
    }
}

#[test]
fn a_judged_answer_records_its_semantic_score() {
    let dir = vault("telemetry-semantic", None);
    let rows = rate_with(&dir, timing(Some(0.8)));
    assert_eq!(rows.len(), 1, "{rows:?}");
    let s = rows[0].semantic_score.expect("a semantic score");
    assert!((s - 0.8).abs() < 1e-6, "{s}");
}

#[test]
fn dwell_and_latency_are_stored_apart_and_the_complexity_is_the_word_count() {
    let dir = vault("telemetry-dwell", None);
    let rows = rate_with(&dir, timing(None));
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].latency_ms, 1_200);
    assert_eq!(rows[0].dwell_ms, 4_800);
    assert_eq!(rows[0].text_complexity, 17);
    assert!(
        rows[0].semantic_score.is_none(),
        "flip-mode answer has nothing to score"
    );
}
