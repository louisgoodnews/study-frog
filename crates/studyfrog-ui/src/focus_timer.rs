//! The focus timer (STORY-0112, `docs/specs/focus-timer.md`): work and break phases on the shell's
//! monotonic clock (`State::now`), so every step is deterministic. Pure.

/// The phase the timer is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Focused work.
    #[default]
    Work,
    /// The break after a work block.
    ShortBreak,
    /// The longer break after every fourth work block.
    LongBreak,
}

/// Minutes of work, short break and long break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    /// Work minutes.
    pub work: u16,
    /// Short break minutes.
    pub short: u16,
    /// Long break minutes (after every fourth block).
    pub long: u16,
}

/// The standard presets: 25/5, 50/10 and 90/15.
pub const PRESETS: [Preset; 3] = [
    Preset {
        work: 25,
        short: 5,
        long: 15,
    },
    Preset {
        work: 50,
        short: 10,
        long: 20,
    },
    Preset {
        work: 90,
        short: 15,
        long: 30,
    },
];

/// Work blocks before a long break.
pub const LONG_EVERY: u32 = 4;

/// A finished work block, to be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusLog {
    /// Wall clock at the start (Unix milliseconds).
    pub started_ms: i64,
    /// Wall clock at the end.
    pub ended_ms: i64,
    /// Minutes focused (pauses excluded).
    pub minutes: u32,
    /// The stack folder chosen.
    pub stack: Option<String>,
}

/// The timer.
#[derive(Debug, Clone, PartialEq)]
pub struct Focus {
    /// Minutes of the phases.
    pub preset: Preset,
    /// The current phase.
    pub phase: Phase,
    /// `State::now` when the phase last started or resumed; `None` while stopped or paused.
    pub since: Option<f64>,
    /// Seconds of the phase that ran before `since`.
    pub banked: f64,
    /// Wall clock when the current work block first started.
    pub started_ms: Option<i64>,
    /// Work blocks finished.
    pub blocks: u32,
    /// The stack folder the focus counts for.
    pub stack: Option<String>,
    /// A finished block waiting for the user's choice (start the break, log it in the diary).
    pub done: Option<FocusLog>,
    /// What the last record said (the session it counted towards).
    pub recorded: Option<String>,
}

impl Default for Focus {
    fn default() -> Self {
        focus_new()
    }
}

/// A stopped timer at the first preset.
pub fn focus_new() -> Focus {
    Focus {
        preset: PRESETS[0],
        phase: Phase::Work,
        since: None,
        banked: 0.0,
        started_ms: None,
        blocks: 0,
        stack: None,
        done: None,
        recorded: None,
    }
}

/// Seconds the phase lasts.
pub fn phase_seconds(f: &Focus) -> f64 {
    let min = match f.phase {
        Phase::Work => f.preset.work,
        Phase::ShortBreak => f.preset.short,
        Phase::LongBreak => f.preset.long,
    };
    f64::from(min) * 60.0
}

/// Seconds of the phase that have run at `now`.
pub fn elapsed(f: &Focus, now: f64) -> f64 {
    f.banked + f.since.map_or(0.0, |t| (now - t).max(0.0))
}

/// Whole seconds left at `now`.
pub fn remaining(f: &Focus, now: f64) -> u32 {
    (phase_seconds(f) - elapsed(f, now)).max(0.0).ceil() as u32
}

/// The timer is counting.
pub fn is_running(f: &Focus) -> bool {
    f.since.is_some()
}

/// The timer is neither counting nor part-way through a phase.
pub fn is_idle(f: &Focus) -> bool {
    f.since.is_none() && f.banked == 0.0 && f.phase == Phase::Work
}

/// `mm:ss`.
pub fn clock(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

/// Start or resume the phase.
pub fn start(f: &mut Focus, now: f64, wall_ms: i64) {
    if f.since.is_none() {
        f.since = Some(now);
    }
    if f.phase == Phase::Work && f.started_ms.is_none() {
        f.started_ms = Some(wall_ms);
    }
}

/// Pause, keeping what has run.
pub fn pause(f: &mut Focus, now: f64) {
    f.banked = elapsed(f, now);
    f.since = None;
}

/// Back to a stopped work phase; finished blocks are kept.
pub fn reset(f: &mut Focus) {
    f.phase = Phase::Work;
    f.since = None;
    f.banked = 0.0;
    f.started_ms = None;
}

/// End the break now and start the next work block.
pub fn skip_break(f: &mut Focus, now: f64, wall_ms: i64) {
    if f.phase != Phase::Work {
        reset(f);
        start(f, now, wall_ms);
    }
}

/// Choose the phase lengths; a running timer keeps running with the new length.
pub fn set_preset(f: &mut Focus, p: Preset) {
    f.preset = Preset {
        work: p.work.clamp(1, 600),
        short: p.short.clamp(1, 120),
        long: p.long.clamp(1, 240),
    };
}

/// Advance to `now`: a finished work block becomes a break (stopped, waiting for the user) and is
/// returned to be recorded; a finished break becomes a stopped work phase.
pub fn tick(f: &mut Focus, now: f64, wall_ms: i64) -> Option<FocusLog> {
    if !is_running(f) || elapsed(f, now) < phase_seconds(f) {
        return None;
    }
    let was = f.phase;
    let worked = elapsed(f, now);
    let started = f.started_ms.unwrap_or(wall_ms - (worked * 1000.0) as i64);
    reset(f);
    if was != Phase::Work {
        return None;
    }
    f.blocks += 1;
    f.phase = if f.blocks.is_multiple_of(LONG_EVERY) {
        Phase::LongBreak
    } else {
        Phase::ShortBreak
    };
    let log = FocusLog {
        started_ms: started,
        ended_ms: wall_ms,
        minutes: (worked / 60.0).round() as u32,
        stack: f.stack.clone(),
    };
    f.done = Some(log.clone());
    Some(log)
}

pub use crate::focus_ui::FocusMsg;
