//! Opening a vault and the first screen: reading the saved settings, the vault dialog's invitation, opening a path and
//! what the vault's opening shows. Pure.

use studyfrog_core::settings::with_opened_vault;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, Screen, State, VaultInfo};

pub(crate) fn notice(kind: NoticeKind, text: impl Into<String>) -> Option<Notice> {
    Some(Notice {
        kind,
        text: text.into(),
    })
}

/// The in-app invitation to choose a vault, shown instead of asking for a path up front.
fn vault_toast() -> crate::modal::Toast {
    crate::modal::Toast {
        title: "Set up your vault".to_owned(),
        detail: "Choose the folder StudyFrog keeps your notes in.".to_owned(),
        action: crate::modal::ToastAction::OpenVaultDialog,
        expires_ms: None,
    }
}

pub(crate) fn started(
    state: State,
    app: studyfrog_core::settings::AppSettings,
    error: Option<String>,
) -> (State, Vec<Effect>) {
    let notice =
        error.and_then(|e| notice(NoticeKind::Error, format!("App settings not used: {e}")));
    let last = app.vault.clone();
    let mut library = state.library.clone();
    if let Some(text) = &app.library_columns {
        library.columns = crate::library_ui::columns_from_text(text);
    }
    let state = crate::appearance_ui::on_start(State {
        path_input: last.clone().unwrap_or_default(),
        app,
        notice,
        library,
        ..state
    });
    match last {
        Some(path) => open(state, path),
        None => (
            State {
                toasts: vec![vault_toast()],
                ..state
            },
            Vec::new(),
        ),
    }
}

pub(crate) fn open_typed_path(state: State) -> (State, Vec<Effect>) {
    let path = state.path_input.trim().to_owned();
    if path.is_empty() {
        let notice = notice(NoticeKind::Error, "Enter the path of a vault folder first.");
        return (State { notice, ..state }, Vec::new());
    }
    open(state, path)
}

pub(crate) fn open(state: State, path: String) -> (State, Vec<Effect>) {
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

pub(crate) fn vault_opened(
    state: State,
    result: Result<VaultInfo, String>,
) -> (State, Vec<Effect>) {
    match result {
        Ok(info) => {
            let app = with_opened_vault(&state.app, &info.root);
            let state = crate::tutorial::offer(State {
                screen: Screen::Home,
                home: None,
                reminder_ticks: 0,
                modal: None,
                toasts: Vec::new(),
                vault: Some(info),
                app,
                opening: false,
                notice: None,
                ..state
            });
            let (state, go) = crate::appearance_ui::go_start(state);
            let mut effects = vec![
                Effect::SaveAppSettings(state.app.clone()),
                Effect::LoadHome,
                Effect::LoadSubjects,
            ];
            effects.extend(go);
            (state, effects)
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
