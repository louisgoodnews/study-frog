//! Saving the open document by itself (TASK-0200). Pure.
//!
//! An edit is saved when the user finishes a token (typed a whitespace or line break), when something other
//! than text changed, or at the latest [`SAVE_AFTER_MS`] after the first unsaved edit. A pause alone saves
//! nothing. At most one save runs; edits made meanwhile are saved right after it. Leaving the editor, opening
//! another item and exiting save first and then carry on ([`flush_before`]).

use crate::edit_form::apply_form;
use crate::editor::{is_dirty, notice};
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{NoticeKind, State};
use studyfrog_service::document::Saved;

/// Unsaved text is saved at the latest this long after the first edit, in milliseconds.
pub const SAVE_AFTER_MS: i64 = 5_000;
/// The start of the service's refusal when the file changed outside StudyFrog.
const CONFLICT_MARK: &str = "was changed outside StudyFrog";

/// True when typing turned `old` into `new` and the last character put in is a whitespace or a line break, so a
/// token (a word) is finished. A deletion never finishes one.
pub fn token_done(old: &str, new: &str) -> bool {
    let (old_len, new_len) = (old.chars().count(), new.chars().count());
    let prefix = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let room = old_len.min(new_len) - prefix;
    let suffix = old
        .chars()
        .rev()
        .zip(new.chars().rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();
    let end = new_len - suffix;
    end > prefix && new.chars().nth(end - 1).is_some_and(char::is_whitespace)
}

/// Save now, or right after the save that is running.
pub fn save(state: State) -> (State, Vec<Effect>) {
    let Some(e) = &state.editor else {
        return (state, Vec::new());
    };
    if !is_dirty(e) {
        return (state, Vec::new());
    }
    if e.saving.is_some() {
        let mut state = state;
        if let Some(e) = &mut state.editor {
            e.again = true;
        }
        return (state, Vec::new());
    }
    match apply_form(&e.doc, &e.form) {
        Ok(doc) => {
            let effect = Effect::SaveDocument {
                path: e.path.clone(),
                doc: doc.clone(),
                hash: e.hash,
            };
            let mut state = state;
            if let Some(e) = &mut state.editor {
                e.saving = Some((doc, e.form.clone()));
            }
            (state, vec![effect])
        }
        Err(reason) => notice(state, NoticeKind::Error, reason),
    }
}

/// Note that the form changed: start the clock of the first unsaved edit, or stop it when nothing differs.
pub fn edited(state: State, finished: bool) -> (State, Vec<Effect>) {
    let mut state = state;
    let now = state.wall_ms;
    let Some(e) = &mut state.editor else {
        return (state, Vec::new());
    };
    if !is_dirty(e) {
        e.dirty_since = None;
        return (state, Vec::new());
    }
    e.dirty_since.get_or_insert(now);
    if finished && !e.conflict {
        save(state)
    } else {
        (state, Vec::new())
    }
}

/// The once-a-second check for the 5 second limit and for the retry of a failed save.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    let now = state.wall_ms;
    let due = state.editor.as_ref().is_some_and(|e| {
        is_dirty(e)
            && e.saving.is_none()
            && !e.conflict
            && e.dirty_since.is_some_and(|t| now - t >= SAVE_AFTER_MS)
    });
    if due {
        save(state)
    } else {
        (state, Vec::new())
    }
}

/// The answer of a save.
pub fn saved(state: State, result: Result<Saved, String>) -> (State, Vec<Effect>) {
    let mut state = state;
    let now = state.wall_ms;
    let Some(e) = &mut state.editor else {
        return (state, Vec::new());
    };
    let Some((doc, form)) = e.saving.take() else {
        return (state, Vec::new());
    };
    match result {
        Ok(saved) => {
            (e.doc, e.hash, e.original) = (doc, saved.hash, form);
            (e.fail, e.conflict, e.saves) = (None, false, e.saves + 1);
            e.dirty_since = is_dirty(e).then_some(now);
            let more = std::mem::take(&mut e.again) || e.after.is_some();
            state.vault = Some(saved.info);
            carry_on(state, more)
        }
        Err(reason) => {
            e.conflict = reason.contains(CONFLICT_MARK);
            e.dirty_since = (!e.conflict).then_some(now);
            e.after = None;
            e.fail = Some(reason.clone());
            notice(state, NoticeKind::Error, format!("Not saved: {reason}"))
        }
    }
}

/// After a save: save what was typed meanwhile, or do what was waiting for the file to be safe.
fn carry_on(state: State, more: bool) -> (State, Vec<Effect>) {
    let dirty = state.editor.as_ref().is_some_and(is_dirty);
    if dirty && more {
        return save(state);
    }
    let mut state = state;
    let waiting = state.editor.as_mut().and_then(|e| e.after.take());
    match (dirty, waiting) {
        (false, Some(then)) => {
            state.replay = Some(then);
            (state, Vec::new())
        }
        (_, waiting) => {
            if let Some(e) = &mut state.editor {
                e.after = waiting;
            }
            (state, Vec::new())
        }
    }
}

/// Before leaving: when there is something unsaved or a save is running, save and then replay `then`.
/// `None` means there is nothing to wait for and the state is untouched; `Some` is the work to wait for.
pub fn flush_before(state: State, then: Msg) -> (State, Option<Vec<Effect>>) {
    let busy = state
        .editor
        .as_ref()
        .is_some_and(|e| is_dirty(e) || e.saving.is_some());
    if !busy {
        return (state, None);
    }
    if state.editor.as_ref().is_some_and(|e| e.conflict) {
        let why = "The file changed outside StudyFrog. Use Reload from disk or Keep mine first.";
        let (state, effects) = notice(state, NoticeKind::Error, why);
        return (state, Some(effects));
    }
    let mut state = state;
    if let Some(e) = &mut state.editor {
        e.after = Some(Box::new(then));
    }
    let (state, effects) = save(state);
    (state, Some(effects))
}
