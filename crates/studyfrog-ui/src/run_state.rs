//! State of a run in progress. Plain data; the engine session is part of it.

use std::collections::{HashMap, HashSet};

use studyfrog_core::event::{Latency, RunEvent};
use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, parse_blocks};
use studyfrog_md::choice::ChoiceBlock;
use studyfrog_rehearsal::reading::{ReadingParams, reading_params_default};
use studyfrog_rehearsal::session::Session;
pub use studyfrog_service::compare::Scored;

pub mod choice;
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
    /// The post-rehearsal diary prompt, if shown or active.
    pub diary_prompt: Option<crate::diary_prompt::RunDiaryPrompt>,
    /// Exam mode: unscored answers, grading and the result.
    pub exam: crate::exam_ui::ExamProgress,
    /// Whether the comments drawer is expanded on screen (STORY-0140).
    pub show_comments: bool,
    /// Draft comment input in the drawer.
    pub comment_draft: String,
    /// The case study on screen, when the item is one: ticked options and the check.
    pub case_run: crate::setup::case::run::CaseRun,
    /// The choice blocks of the loaded items that are choice items in an answer run (STORY-0222).
    pub choices: HashMap<Uid, ChoiceBlock>,
    /// The options picked on the current choice item.
    pub choice: choice::ChoiceSelection,
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
        diary_prompt: None,
        exam: crate::exam_ui::ExamProgress::default(),
        show_comments: false,
        comment_draft: String::new(),
        case_run: Default::default(),
        choices: HashMap::new(),
        choice: Default::default(),
    }
}

/// Milliseconds of engine time at `now` (seconds), never before `floor`.
pub fn engine_ms(now: f64, started_time: f64, floor: u32) -> u32 {
    let ms = ((now - started_time) * 1000.0).max(0.0);
    (ms.min(f64::from(u32::MAX)) as u32).max(floor)
}

/// Work on an answer in a run: the typed answer is scored, a case check's score is recorded like a
/// typed answer's.
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerEffect {
    /// Compare a typed answer with the item's reference. `answer` is what was typed, and the words to
    /// compare it with when not the whole back (a cloze gap); `words_only` compares by words only.
    Score {
        /// The run the answer belongs to.
        run: studyfrog_core::uid::Uid,
        /// The item answered.
        item: studyfrog_core::uid::Uid,
        /// What was typed, and the reference words.
        answer: (String, Option<String>),
        /// Compare by word matching only, whatever is installed.
        words_only: bool,
    },
    /// Store a case check's score for the item, as a typed answer's score is stored.
    Record {
        /// The run.
        run: studyfrog_core::uid::Uid,
        /// The item.
        item: studyfrog_core::uid::Uid,
        /// The score and the rating it suggests.
        scored: Box<studyfrog_service::compare::Scored>,
    },
    /// Grade a choice answer with the service's rules and store it as a typed answer's score (STORY-0222).
    Choice {
        /// The run.
        run: studyfrog_core::uid::Uid,
        /// The choice item answered.
        item: studyfrog_core::uid::Uid,
        /// One flag per option, in the order written.
        picked: Vec<bool>,
        /// The grading scheme the settings choose (STORY-0231).
        scheme: studyfrog_core::choice_grading::ChoiceGrading,
    },
}
