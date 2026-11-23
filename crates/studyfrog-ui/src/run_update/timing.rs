//! The timing of a rating, as the run measured it: the telemetry and familiarity inputs of
//! `record_step`. Pure.

use studyfrog_core::event::{Latency, LatencyKind};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::reading::{expected_ms, item_records, word_count};
use studyfrog_service::rehearse::RatingTiming;

use crate::run_state::RunView;

/// The timing of a rating: how long the answer was on screen (familiarity), the first response and
/// the dwell (telemetry), the item's size and the typed answer's score, if it was judged.
pub(crate) fn rating_timing(
    run: &RunView,
    item: Uid,
    (dwell, judged): (Option<u32>, Option<f32>),
) -> Option<RatingTiming> {
    let record = item_records(&run.events)
        .into_iter()
        .find(|r| r.item == item)?;
    let answer_ms = record.rated.and(record.read_ms)?;
    let content = run.content.get(&item)?;
    let words = word_count(&content.answer);
    if words == 0 {
        return None;
    }
    Some(RatingTiming {
        answer_ms,
        expected_ms: expected_ms(words, &run.reading),
        latency_ms: first_response_ms(&run.latencies, item)?,
        dwell_ms: dwell?,
        text_words: u32::try_from(word_count(&content.prompt) + words).unwrap_or(u32::MAX),
        semantic: judged,
    })
}

/// Milliseconds from the item being shown to its first response (the last flip or submitted answer).
fn first_response_ms(latencies: &[Latency], item: Uid) -> Option<u32> {
    latencies
        .iter()
        .rev()
        .find(|l| l.item == item && l.kind != LatencyKind::Skip)
        .map(|l| l.ms)
}
