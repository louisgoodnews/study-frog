//! The focus timer in the window (STORY-0112): its messages, the tick, recording a finished block and the
//! diary entry for it. Pure; edits the owned state in place.

use studyfrog_service::focus::{FocusDone, FocusRecorded};

use crate::effect::Effect;
use crate::focus_timer::{self as ft, FocusLog, Phase, Preset};
use crate::recall_types::{RecallEffect, RecallMsg};
use crate::recall_ui::{effect, notice};
use crate::state::{NoticeKind, Screen, State};

/// Something happened to the focus timer.
#[derive(Debug, Clone, PartialEq)]
pub enum FocusMsg {
    /// Start or resume.
    Start,
    /// Pause.
    Pause,
    /// Stop and go back to a fresh work phase.
    Reset,
    /// End the break and start working.
    SkipBreak,
    /// Start the break after a finished block (from the completion card).
    StartBreak,
    /// Use these phase lengths.
    Preset(Preset),
    /// Count the focus for this stack folder (or none).
    Stack(Option<String>),
    /// Open the diary with an entry for the finished block.
    LogDiary,
    /// Close the completion card.
    Dismiss,
    /// The block was recorded.
    Recorded(Result<FocusRecorded, String>),
}

/// The message for the timer.
pub fn msg(m: FocusMsg) -> crate::msg::Msg {
    crate::msg::Msg::Recall(RecallMsg::Focus(m))
}

fn record(log: FocusLog) -> Effect {
    effect(RecallEffect::LogFocus(FocusDone {
        started_ms: log.started_ms,
        ended_ms: log.ended_ms,
        minutes: log.minutes,
        stack: log.stack,
    }))
}

/// Advance the timer on the clock tick; a finished block is recorded.
pub fn tick(mut state: State) -> (State, Vec<Effect>) {
    let (now, wall) = (state.now, state.wall_ms);
    let was = state.recall.focus.phase;
    let done = ft::tick(&mut state.recall.focus, now, wall);
    if was != Phase::Work && state.recall.focus.phase == Phase::Work {
        notice(
            &mut state,
            NoticeKind::Info,
            "The break is over.".to_owned(),
        );
    }
    let open = state.vault.is_some();
    (
        state,
        done.filter(|_| open).map(record).into_iter().collect(),
    )
}

/// Handle one message.
pub fn update(state: &mut State, m: FocusMsg) -> Vec<Effect> {
    let (now, wall) = (state.now, state.wall_ms);
    let f = &mut state.recall.focus;
    match m {
        FocusMsg::Start => ft::start(f, now, wall),
        FocusMsg::Pause => ft::pause(f, now),
        FocusMsg::Reset => ft::reset(f),
        FocusMsg::SkipBreak => ft::skip_break(f, now, wall),
        FocusMsg::StartBreak => {
            f.done = None;
            if f.phase != Phase::Work {
                ft::start(f, now, wall);
            }
        }
        FocusMsg::Preset(p) => ft::set_preset(f, p),
        FocusMsg::Stack(s) => f.stack = s,
        FocusMsg::Dismiss => f.done = None,
        FocusMsg::LogDiary => log_diary(state),
        FocusMsg::Recorded(r) => recorded(state, r),
    }
    Vec::new()
}

fn recorded(state: &mut State, r: Result<FocusRecorded, String>) {
    match r {
        Ok(rec) => {
            let towards = rec.session.map_or(String::new(), |(_, title, _)| {
                format!(" towards \"{title}\"")
            });
            let text = format!(
                "Focus block of {} min recorded{towards}; {} min today.",
                rec.minutes, rec.today
            );
            state.recall.focus.recorded = Some(text.clone());
            notice(state, NoticeKind::Info, text);
        }
        Err(e) => notice(
            state,
            NoticeKind::Error,
            format!("Could not record the focus block: {e}"),
        ),
    }
}

/// The diary editor with an entry for the finished block: today, its minutes and stack.
fn log_diary(state: &mut State) {
    let Some(done) = state.recall.focus.done.take() else {
        return;
    };
    let mut draft = crate::diary_types::new_draft(crate::view::date_picker::today_of(state));
    draft.study_minutes = Some(done.minutes);
    draft.stacks = done.stack.unwrap_or_default();
    draft.content = format!("Focus session of {} minutes.\n\n", done.minutes);
    draft.is_dirty = true;
    state.diary.active_entry = Some(draft);
    state.screen = Screen::Diary;
}
