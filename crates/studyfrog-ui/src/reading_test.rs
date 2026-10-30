//! The reading-speed test: read a fixed text, say when done, keep the measured speed. Pure; time
//! enters only through `State::now`.

use studyfrog_core::settings::{AnswerEngine, AppSettings};
use studyfrog_rehearsal::reading::{DEFAULT_WPM, measured_wpm, word_count};

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// The text to read: standard placeholder text, so nothing in it has to be understood or recalled.
pub const TEXT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit \
esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa \
qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit \
voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis \
et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit \
aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.";

/// Where the test is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TestPhase {
    /// Not running.
    #[default]
    Idle,
    /// The text is on screen since this `State::now`.
    Reading(f64),
    /// Finished; `None` when the time was not plausible for reading.
    Result(Option<u32>),
}

/// State of the reading-speed test.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ReadingTest {
    /// Current phase.
    pub phase: TestPhase,
}

/// What the user can do in the test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingTestMsg {
    /// Show the text and start the clock.
    Start,
    /// "I'm done" was activated.
    Finish,
    /// Keep the measured speed.
    Save,
    /// Throw the result away.
    Discard,
    /// Go back to the default speed.
    Reset,
}

/// The speed in use: the measured one, else the default.
pub fn current_wpm(state: &State) -> u32 {
    state.app.reading_wpm.unwrap_or(DEFAULT_WPM)
}

fn phase(state: State, phase: TestPhase) -> (State, Vec<Effect>) {
    (
        State {
            reading_test: ReadingTest { phase },
            ..state
        },
        Vec::new(),
    )
}

fn store(state: State, wpm: Option<u32>, text: String) -> (State, Vec<Effect>) {
    let app = AppSettings {
        reading_wpm: wpm,
        ..state.app.clone()
    };
    let notice = Some(Notice {
        kind: NoticeKind::Info,
        text,
    });
    let effects = vec![Effect::SaveAppSettings(app.clone())];
    let state = State {
        app,
        notice,
        reading_test: ReadingTest::default(),
        ..state
    };
    (state, effects)
}

/// Handle the test's messages.
pub fn update(state: State, msg: ReadingTestMsg) -> (State, Vec<Effect>) {
    let current = state.reading_test.phase;
    match (msg, current) {
        (ReadingTestMsg::Start, _) => {
            let now = state.now;
            phase(state, TestPhase::Reading(now))
        }
        (ReadingTestMsg::Finish, TestPhase::Reading(since)) => {
            let ms = ((state.now - since).max(0.0) * 1000.0).min(f64::from(u32::MAX)) as u32;
            let wpm = measured_wpm(word_count(TEXT), ms);
            phase(state, TestPhase::Result(wpm))
        }
        (ReadingTestMsg::Save, TestPhase::Result(Some(w))) => store(
            state,
            Some(w),
            format!("Reading speed saved: {w} words per minute."),
        ),
        (ReadingTestMsg::Reset, _) => store(
            state,
            None,
            format!("Reading speed reset to the default of {DEFAULT_WPM} words per minute."),
        ),
        (ReadingTestMsg::Discard, _) => phase(state, TestPhase::Idle),
        _ => (state, Vec::new()),
    }
}

/// Remember how typed answers should be compared.
pub fn set_answer_engine(state: State, engine: AnswerEngine) -> (State, Vec<Effect>) {
    let app = AppSettings {
        answer_engine: engine,
        ..state.app.clone()
    };
    let effects = vec![Effect::SaveAppSettings(app.clone())];
    (State { app, ..state }, effects)
}
