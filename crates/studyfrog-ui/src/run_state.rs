//! State of a run in progress. Plain data; the engine session is part of it.

use std::collections::{HashMap, HashSet};

use studyfrog_core::event::{Latency, RunEvent};
use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, parse_blocks};
use studyfrog_rehearsal::reading::{ReadingParams, reading_params_default};
use studyfrog_rehearsal::session::Session;
pub use studyfrog_service::compare::Scored;
pub use studyfrog_service::content::ItemContent;

/// An item's prompt and answer as Markdown blocks.
pub fn blocks_of(content: &ItemContent) -> (Vec<Block>, Vec<Block>) {
    (parse_blocks(&content.prompt), parse_blocks(&content.answer))
}

/// How many items ahead of the current one are loaded.
pub const LOOKAHEAD: usize = 3;

/// A run on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct RunView {
    /// The engine state.
    pub session: Session,
    /// Content of items loaded so far.
    pub content: HashMap<Uid, ItemContent>,
    /// The prompt and the answer of each loaded item, parsed once as Markdown.
    pub blocks: HashMap<Uid, (Vec<Block>, Vec<Block>)>,
    /// Items whose content was already requested.
    pub requested: HashSet<Uid>,
    /// Every event so far (for the summary).
    pub events: Vec<RunEvent>,
    /// Every latency so far.
    pub latencies: Vec<Latency>,
    /// The answer being typed (answer mode).
    pub answer: String,
    /// The reading speed assumptions in force when the run started.
    pub reading: ReadingParams,
    /// Items whose rating was replaced by the suggested one after the run.
    pub applied: HashSet<Uid>,
    /// The latest comparison of each item's answer, kept for the review after the run.
    pub scores: HashMap<Uid, Scored>,
    /// How the submitted answer compared with the reference, once scored.
    pub scored: Option<Scored>,
    /// `State::now` when the run started; engine time is measured from here.
    pub started_time: f64,
}

/// A fresh view of a not-yet-started session.
pub fn run_view_new(session: Session, started_time: f64) -> RunView {
    RunView {
        session,
        content: HashMap::new(),
        blocks: HashMap::new(),
        requested: HashSet::new(),
        events: Vec::new(),
        latencies: Vec::new(),
        answer: String::new(),
        reading: reading_params_default(),
        applied: HashSet::new(),
        scores: HashMap::new(),
        scored: None,
        started_time,
    }
}

/// Milliseconds of engine time at `now` (seconds), never before `floor`.
pub fn engine_ms(now: f64, started_time: f64, floor: u32) -> u32 {
    let ms = ((now - started_time) * 1000.0).max(0.0);
    (ms.min(f64::from(u32::MAX)) as u32).max(floor)
}
