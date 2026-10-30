//! Run events and derived latency records.
//!
//! Events are stored as an array of structs for now; whether a struct-of-arrays layout pays off
//! is decided by the M3 rehearsal benchmark (EPIC-003 open question).

use crate::review::{Rating, rating_from_value, rating_value};
use crate::uid::Uid;

/// What happened. Payloads are only where the spec needs them.
#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    /// Queue fixed, run started.
    RunStarted,
    /// Prompt displayed; starts the item timer.
    ItemShown,
    /// User skipped the item.
    ItemSkipped,
    /// Answer revealed.
    ItemFlipped,
    /// Free-text answer submitted (stored verbatim).
    AnswerSubmitted(String),
    /// Rating chosen; feeds SRS.
    ItemRated(Rating),
    /// Run paused.
    RunPaused,
    /// Run resumed.
    RunResumed,
    /// Normal end.
    RunFinished,
    /// Early exit or crash recovery.
    RunAborted,
}

/// One timestamped record in a run's append-only log.
#[derive(Debug, Clone, PartialEq)]
pub struct RunEvent {
    /// Owning run.
    pub run: Uid,
    /// Position in the run's log, starting at 0.
    pub seq: u32,
    /// Monotonic milliseconds since run start (about 49 days of range).
    pub t_ms: u32,
    /// The item concerned; `None` for run-level events.
    pub item: Option<Uid>,
    /// What happened.
    pub kind: EventKind,
}

/// Which interaction ended the item timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LatencyKind {
    /// Time-to-skip.
    Skip,
    /// Time-to-flip.
    Flip,
    /// Time-to-answer.
    Answer,
}

/// A latency derived from events (paused time already excluded), in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Latency {
    /// The item.
    pub item: Uid,
    /// Which interaction.
    pub kind: LatencyKind,
    /// Duration in milliseconds.
    pub ms: u32,
}

/// Persisted name of an event kind (payloads are stored separately).
pub fn event_name(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::RunStarted => "run_started",
        EventKind::ItemShown => "item_shown",
        EventKind::ItemSkipped => "item_skipped",
        EventKind::ItemFlipped => "item_flipped",
        EventKind::AnswerSubmitted(_) => "answer_submitted",
        EventKind::ItemRated(_) => "item_rated",
        EventKind::RunPaused => "run_paused",
        EventKind::RunResumed => "run_resumed",
        EventKind::RunFinished => "run_finished",
        EventKind::RunAborted => "run_aborted",
    }
}

/// True for events that concern a single item (they carry `item: Some(_)`).
pub fn is_item_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::ItemShown
            | EventKind::ItemSkipped
            | EventKind::ItemFlipped
            | EventKind::AnswerSubmitted(_)
            | EventKind::ItemRated(_)
    )
}

/// The payload stored next to an event name, if the kind has one (answer text, or the rating as 1-4).
pub fn event_payload(kind: &EventKind) -> Option<String> {
    match kind {
        EventKind::AnswerSubmitted(text) => Some(text.clone()),
        EventKind::ItemRated(rating) => Some(rating_value(*rating).to_string()),
        _ => None,
    }
}

/// Rebuild an event kind from its persisted name and payload; `None` if they do not fit.
pub fn event_from_parts(name: &str, payload: Option<&str>) -> Option<EventKind> {
    match (name, payload) {
        ("run_started", None) => Some(EventKind::RunStarted),
        ("item_shown", None) => Some(EventKind::ItemShown),
        ("item_skipped", None) => Some(EventKind::ItemSkipped),
        ("item_flipped", None) => Some(EventKind::ItemFlipped),
        ("answer_submitted", Some(text)) => Some(EventKind::AnswerSubmitted(text.to_owned())),
        ("item_rated", Some(v)) => v
            .parse()
            .ok()
            .and_then(rating_from_value)
            .map(EventKind::ItemRated),
        ("run_paused", None) => Some(EventKind::RunPaused),
        ("run_resumed", None) => Some(EventKind::RunResumed),
        ("run_finished", None) => Some(EventKind::RunFinished),
        ("run_aborted", None) => Some(EventKind::RunAborted),
        _ => None,
    }
}
