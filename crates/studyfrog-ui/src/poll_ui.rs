//! Checking the vault for changes while the app is open. Pure.

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, Screen, State, VaultInfo};

fn notice(kind: NoticeKind, text: impl Into<String>) -> Option<Notice> {
    Some(Notice {
        kind,
        text: text.into(),
    })
}

/// About once a second: check the open vault for changes unless a check is running.
pub fn tick(state: State) -> (State, Vec<Effect>) {
    if state.vault.is_some() && !state.polling {
        (
            State {
                polling: true,
                ..state
            },
            vec![Effect::PollVault],
        )
    } else {
        (state, Vec::new())
    }
}

/// A change check finished: refresh what depends on the vault.
pub fn vault_polled(
    state: State,
    result: Result<Option<VaultInfo>, String>,
) -> (State, Vec<Effect>) {
    let state = State {
        polling: false,
        ..state
    };
    match result {
        Ok(Some(info)) if state.vault.is_some() => {
            let mut effects = crate::browse::reload(&state.browser.selection);
            effects.push(crate::pdf_library_ui::load());
            let state = State {
                vault: Some(info),
                ..state
            };
            if state.screen == Screen::Graph {
                effects.push(Effect::LoadGraph);
            }
            if state.screen != Screen::Rehearse {
                return (state, effects);
            }
            let (state, preview) = crate::setup::refresh_preview(state);
            effects.extend(preview);
            (state, effects)
        }
        Ok(_) => (state, Vec::new()),
        Err(e) => {
            let notice = notice(NoticeKind::Error, format!("Checking the vault failed: {e}"));
            (State { notice, ..state }, Vec::new())
        }
    }
}
