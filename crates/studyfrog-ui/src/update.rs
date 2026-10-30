//! The pure update function: `(State, Msg) -> (State, Vec<Effect>)`. No I/O, no clock, no egui.

use studyfrog_core::settings::with_opened_vault;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::{Notice, NoticeKind, Screen, State, VaultInfo};

fn notice(kind: NoticeKind, text: impl Into<String>) -> Option<Notice> {
    Some(Notice {
        kind,
        text: text.into(),
    })
}

/// Advance the state by one message.
pub fn update(state: State, msg: Msg) -> (State, Vec<Effect>) {
    match msg {
        Msg::Started { app, error } => started(state, app, error),
        Msg::PathEdited(text) => (
            State {
                path_input: text,
                ..state
            },
            Vec::new(),
        ),
        Msg::OpenVaultClicked => open_typed_path(state),
        Msg::OpenRecent(path) => open(
            State {
                path_input: path.clone(),
                ..state
            },
            path,
        ),
        Msg::VaultOpened(result) => vault_opened(state, result),
        Msg::Navigate(screen) => navigate(state, screen),
        Msg::CloseVault => (
            State {
                vault: None,
                screen: Screen::Welcome,
                polling: false,
                ..state
            },
            Vec::new(),
        ),
        Msg::DismissNotice => (
            State {
                notice: None,
                ..state
            },
            Vec::new(),
        ),
        Msg::Tick => tick(state),
        Msg::VaultPolled(result) => vault_polled(state, result),
        Msg::AppSettingsSaved(result) => settings_saved(state, result),
        browse @ (Msg::Select(_)
        | Msg::SearchEdited(_)
        | Msg::ItemsLoaded(_)
        | Msg::NewStackEdited(_)
        | Msg::CreateStackClicked
        | Msg::DraftEdited(..)
        | Msg::AddCardClicked
        | Msg::Created(_)) => crate::browse::update(state, browse),
        setup @ (Msg::Setup(_) | Msg::PreviewReady(_)) => crate::setup::update(state, setup),
        run @ (Msg::StartRunClicked
        | Msg::RunReady(_)
        | Msg::RunAction(_)
        | Msg::AnswerEdited(_)
        | Msg::ItemLoaded(..)
        | Msg::StepRecorded(_)
        | Msg::LeaveRun) => crate::run_update::update(state, run),
    }
}

fn started(
    state: State,
    app: studyfrog_core::settings::AppSettings,
    error: Option<String>,
) -> (State, Vec<Effect>) {
    let notice =
        error.and_then(|e| notice(NoticeKind::Error, format!("App settings not used: {e}")));
    let last = app.vault.clone();
    let state = State {
        path_input: last.clone().unwrap_or_default(),
        app,
        notice,
        ..state
    };
    match last {
        Some(path) => open(state, path),
        None => (state, Vec::new()),
    }
}

fn open_typed_path(state: State) -> (State, Vec<Effect>) {
    let path = state.path_input.trim().to_owned();
    if path.is_empty() {
        let notice = notice(NoticeKind::Error, "Enter the path of a vault folder first.");
        return (State { notice, ..state }, Vec::new());
    }
    open(state, path)
}

fn open(state: State, path: String) -> (State, Vec<Effect>) {
    if state.opening {
        return (state, Vec::new());
    }
    (
        State {
            opening: true,
            notice: None,
            ..state
        },
        vec![Effect::OpenVault(path)],
    )
}

fn vault_opened(state: State, result: Result<VaultInfo, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(info) => {
            let app = with_opened_vault(&state.app, &info.root);
            let text = format!("Opened {}", info.name);
            let state = State {
                screen: Screen::Library,
                vault: Some(info),
                app: app.clone(),
                opening: false,
                notice: notice(NoticeKind::Info, text),
                ..state
            };
            (state, vec![Effect::SaveAppSettings(app)])
        }
        Err(e) => {
            let notice = notice(NoticeKind::Error, format!("Could not open the vault: {e}"));
            (
                State {
                    opening: false,
                    screen: Screen::Welcome,
                    vault: None,
                    notice,
                    ..state
                },
                Vec::new(),
            )
        }
    }
}

fn navigate(state: State, screen: Screen) -> (State, Vec<Effect>) {
    let allowed = screen == Screen::Welcome || state.vault.is_some();
    if allowed && screen == Screen::Rehearse {
        crate::setup::refresh_preview(State { screen, ..state })
    } else if allowed {
        (State { screen, ..state }, Vec::new())
    } else {
        (state, Vec::new())
    }
}

fn tick(state: State) -> (State, Vec<Effect>) {
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

fn vault_polled(state: State, result: Result<Option<VaultInfo>, String>) -> (State, Vec<Effect>) {
    let state = State {
        polling: false,
        ..state
    };
    match result {
        Ok(Some(info)) if state.vault.is_some() => {
            let mut effects = crate::browse::reload(&state.browser.selection);
            let state = State {
                vault: Some(info),
                ..state
            };
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

fn settings_saved(state: State, result: Result<(), String>) -> (State, Vec<Effect>) {
    match result {
        Ok(()) => (state, Vec::new()),
        Err(e) => {
            let notice = notice(
                NoticeKind::Error,
                format!("Could not remember the vault: {e}"),
            );
            (State { notice, ..state }, Vec::new())
        }
    }
}
