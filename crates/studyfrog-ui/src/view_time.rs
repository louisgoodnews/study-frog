//! How long the item page is open and active, turned into recorded views. Pure: the clock is `state.now`.
//!
//! A visit starts when an item opens on the item page and ends when another item, another screen or no
//! item is shown. Time counts only while the user is active: once nothing has been touched for
//! [`IDLE_SECS`], the clock stops until the next input. Visits shorter than [`MIN_VIEW_MS`] are dropped.

use studyfrog_core::uid::Uid;
use studyfrog_core::views::ViewKind;

use studyfrog_rehearsal::reading::{expected_ms, reading_params, word_count};

use crate::editor::is_dirty;
use crate::effect::Effect;
use crate::state::{Screen, State};

/// Seconds without input after which the user counts as away.
pub const IDLE_SECS: f64 = 60.0;
/// The most one tick may add, so a suspended computer does not count as time on the page.
pub const MAX_STEP_SECS: f64 = 5.0;
/// Visits shorter than this are not recorded (accidental opens).
pub const MIN_VIEW_MS: u32 = 1000;

/// A visit to record: the item, view or edit, the active time and the reading time of its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewRecord {
    /// The item.
    pub item: Uid,
    /// View or edit.
    pub kind: ViewKind,
    /// Active milliseconds.
    pub active_ms: u32,
    /// How long the item's text should take to read, if it has words.
    pub expected_ms: Option<u32>,
}

/// The visit in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewTracker {
    /// The item on the page.
    pub uid: Uid,
    /// The clock at the last tick.
    pub last_tick: f64,
    /// The clock at the last input.
    pub last_input: f64,
    /// Active seconds so far.
    pub active: f64,
    /// The file fingerprint when the page opened; a different one means the item was saved.
    pub opened_hash: u64,
    /// The item was changed or saved during the visit.
    pub edited: bool,
    /// How long the item's text should take to read at the user's speed.
    pub expected_ms: Option<u32>,
}

/// Input happened: the user is here.
pub fn activity(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    let now = state.now;
    if let Some(t) = &mut state.view {
        t.last_input = now;
    }
    (state, Vec::new())
}

/// A tick passed: add the time since the last one if the user is not idle, and notice edits.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    let mut state = state;
    let now = state.now;
    let edited = state
        .editor
        .as_ref()
        .zip(state.view.as_ref())
        .is_some_and(|(e, t)| is_dirty(e) || e.hash != t.opened_hash);
    if let Some(t) = &mut state.view {
        let step = (now - t.last_tick).clamp(0.0, MAX_STEP_SECS);
        if now - t.last_input <= IDLE_SECS {
            t.active += step;
        }
        t.last_tick = now;
        t.edited |= edited;
    }
    (state, Vec::new())
}

/// How long the item's text should take to read at the user's reading speed; `None` for an item without words.
fn reading_time(state: &State, e: &crate::editor::Editor) -> Option<u32> {
    let words: usize = e.form.fields.iter().map(|f| word_count(&f.text)).sum();
    let params = reading_params(state.app.reading_wpm, &state.app.reading);
    (words > 0).then(|| expected_ms(words, &params))
}

/// The editor, when the item page is the screen being shown.
fn on_page(state: &State) -> Option<&crate::editor::Editor> {
    state
        .editor
        .as_ref()
        .filter(|_| state.screen == Screen::Editor)
}

fn tracking(state: &State) -> bool {
    !state.app.views_untracked && state.vault.is_some()
}

fn recorded(t: &ViewTracker) -> Vec<Effect> {
    let active_ms = (t.active * 1000.0) as u32;
    if active_ms < MIN_VIEW_MS {
        return Vec::new();
    }
    let mut out = vec![Effect::RecordView(ViewRecord {
        item: t.uid,
        kind: ViewKind::View,
        active_ms,
        expected_ms: t.expected_ms,
    })];
    if t.edited {
        out.push(Effect::RecordView(ViewRecord {
            item: t.uid,
            kind: ViewKind::Edit,
            active_ms,
            expected_ms: t.expected_ms,
        }));
    }
    out
}

/// Start, end or switch the visit so it matches the item on the page. Call after every update.
pub fn sync(state: State) -> (State, Vec<Effect>) {
    let shown = on_page(&state).map(|e| e.uid);
    let tracked = state.view.as_ref().map(|t| t.uid);
    let enabled = tracking(&state);
    if shown == tracked && (enabled || tracked.is_none()) {
        return (state, Vec::new());
    }
    let mut state = state;
    let effects = match (&state.view, enabled) {
        (Some(t), true) => recorded(t),
        _ => Vec::new(),
    };
    let now = state.now;
    state.view = match (on_page(&state), enabled) {
        (Some(e), true) => Some(ViewTracker {
            uid: e.uid,
            last_tick: now,
            last_input: now,
            active: 0.0,
            opened_hash: e.hash,
            edited: false,
            expected_ms: reading_time(&state, e),
        }),
        _ => None,
    };
    (state, effects)
}
