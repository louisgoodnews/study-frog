//! The active-recall workflows in the window (EPIC-035): routing their messages. Pure; edits the
//! owned state in place to keep the frames small.

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
use crate::recall_types::{RecallEffect, RecallMsg};
use crate::state::{Notice, NoticeKind, State};

/// The effect of a recall workflow.
pub fn effect(e: RecallEffect) -> Effect {
    Effect::Library(LibraryEffect::Recall(e))
}

pub(crate) fn notice(state: &mut State, kind: NoticeKind, text: String) {
    state.notice = Some(Notice { kind, text });
}

/// Handle one message.
pub fn update(mut state: State, msg: RecallMsg) -> (State, Vec<Effect>) {
    let fx = match msg {
        RecallMsg::ExtractPreview(note, body) => preview(&mut state, note, body),
        RecallMsg::ExtractPlanned(r) => planned(&mut state, r),
        RecallMsg::ExtractSync => sync(&mut state),
        RecallMsg::ExtractSynced(r) => synced(&mut state, r),
        RecallMsg::Focus(m) => crate::focus_ui::update(&mut state, m),
        RecallMsg::Contrast(m) => crate::contrast::update(&mut state, m),
        RecallMsg::ExtractClose => {
            state.recall.extract = Default::default();
            Vec::new()
        }
    };
    (state, fx)
}

fn preview(state: &mut State, note: studyfrog_core::uid::Uid, body: String) -> Vec<Effect> {
    let panel = &mut state.recall.extract;
    *panel = Default::default();
    panel.note = Some(note);
    panel.busy = true;
    vec![effect(RecallEffect::PlanExtract(note, body))]
}

fn planned(
    state: &mut State,
    r: Result<Box<studyfrog_service::inline_sync::SyncPlan>, String>,
) -> Vec<Effect> {
    let panel = &mut state.recall.extract;
    panel.busy = false;
    match r {
        Ok(plan) if Some(plan.note) == panel.note => panel.plan = Some(*plan),
        Ok(_) => {}
        Err(e) => panel.error = Some(e),
    }
    Vec::new()
}

fn sync(state: &mut State) -> Vec<Effect> {
    let panel = &mut state.recall.extract;
    match (&panel.plan, panel.busy) {
        (Some(plan), false) => {
            panel.busy = true;
            vec![effect(RecallEffect::ApplyExtract(Box::new(plan.clone())))]
        }
        _ => Vec::new(),
    }
}

fn synced(
    state: &mut State,
    r: Result<Box<studyfrog_service::inline_sync::SyncOutcome>, String>,
) -> Vec<Effect> {
    state.recall.extract.busy = false;
    match r {
        Ok(done) => {
            let done = *done;
            state.recall.extract.done = Some((done.created, done.updated));
            state.recall.extract.plan = None;
            state.vault = Some(done.info);
            state.library.undo = Some(done.undo);
            state.library.stale = true;
            let text = format!(
                "Synced the inline cards: {} new, {} updated. Undo is in the Library.",
                done.created, done.updated
            );
            notice(state, NoticeKind::Info, text);
            crate::browse::reload(&state.browser.selection)
        }
        Err(e) => {
            state.recall.extract.error = Some(e.clone());
            notice(state, NoticeKind::Error, format!("Could not sync: {e}"));
            Vec::new()
        }
    }
}
