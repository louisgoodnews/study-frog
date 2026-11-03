//! "Saved <title>" when the user leaves an item that was saved on the way (TASK-0201). Pure.

use studyfrog_core::uid::Uid;

use crate::modal_types::{Toast, ToastAction};
use crate::state::{Screen, State};

/// How long the toast stays, in milliseconds.
pub const TOAST_MS: i64 = 3_000;

/// The visit to an item: which one, how many times it was saved and its title.
pub type Visit = (Uid, u32, String);

/// The visit now showing in the editor, if any.
pub fn visit(state: &State) -> Option<Visit> {
    let e = state
        .editor
        .as_ref()
        .filter(|_| state.screen == Screen::Editor)?;
    Some((e.uid, e.saves, e.form.title.clone()))
}

/// Raise the toast when the visit `before` is over in `state` and it saved something.
pub fn after(before: Option<Visit>, state: State) -> State {
    let Some((uid, saves, title)) = before.filter(|(_, saves, _)| *saves > 0) else {
        return state;
    };
    let still_there = visit(&state).is_some_and(|(now, _, _)| now == uid);
    if still_there {
        return state;
    }
    let mut state = state;
    if let Some(e) = state.editor.as_mut().filter(|e| e.uid == uid) {
        e.saves = 0;
    }
    state.toasts.push(Toast {
        title: format!("Saved {title}"),
        detail: if saves == 1 {
            "Saved".to_owned()
        } else {
            format!("Saved {saves} times while you edited")
        },
        action: ToastAction::Dismiss,
        expires_ms: Some(state.wall_ms + TOAST_MS),
    });
    state
}

/// Drop the toasts whose time is up.
pub fn expire(state: State) -> State {
    let now = state.wall_ms;
    let mut state = state;
    state
        .toasts
        .retain(|t| t.expires_ms.is_none_or(|at| at > now));
    state
}
