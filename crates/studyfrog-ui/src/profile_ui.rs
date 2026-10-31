//! User profiles: each has its own settings (recent vaults, reading speed) and keys. Switching closes
//! the vault and opens the profile's own last vault. Pure.

use studyfrog_config::profiles::{Profiles, name_problem};
use studyfrog_core::settings::AppSettings;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, Screen, State};

/// The profile menu's state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfileState {
    /// The profiles and the one in use.
    pub list: Profiles,
    /// The name typed for a new profile.
    pub draft: String,
    /// The profile asked to be deleted, waiting for a second click.
    pub confirm_delete: Option<String>,
}

/// Messages about profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileMsg {
    /// The profiles were read at start-up.
    Loaded(Profiles),
    /// The new profile's name changed.
    Draft(String),
    /// Create the typed profile and switch to it.
    Create,
    /// Switch to a profile.
    Switch(String),
    /// Ask to delete a profile (needs a second click).
    AskDelete(String),
    /// Delete the profile that was asked about.
    ConfirmDelete,
    /// Forget the question.
    CancelDelete,
    /// A profile change finished: the profiles and the settings now in use.
    Changed(Result<(Profiles, AppSettings), String>),
}

fn failed(state: State, text: String) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text,
    });
    (State { notice, ..state }, Vec::new())
}

fn with(state: State, f: impl FnOnce(ProfileState) -> ProfileState) -> (State, Vec<Effect>) {
    let profiles = f(state.profiles.clone());
    (State { profiles, ..state }, Vec::new())
}

/// Handle the profile messages.
pub fn update(state: State, msg: ProfileMsg) -> (State, Vec<Effect>) {
    match msg {
        ProfileMsg::Loaded(list) => with(state, |p| ProfileState { list, ..p }),
        ProfileMsg::Draft(draft) => with(state, |p| ProfileState { draft, ..p }),
        ProfileMsg::Create => {
            let name = state.profiles.draft.trim().to_owned();
            match name_problem(&name, &state.profiles.list.names) {
                Some(why) => failed(state, why),
                None => (state, vec![Effect::CreateProfile(name)]),
            }
        }
        ProfileMsg::Switch(name) if name == state.profiles.list.current => (state, Vec::new()),
        ProfileMsg::Switch(name) => (state, vec![Effect::SwitchProfile(name)]),
        ProfileMsg::AskDelete(name) => with(state, |p| ProfileState {
            confirm_delete: Some(name),
            ..p
        }),
        ProfileMsg::CancelDelete => with(state, |p| ProfileState {
            confirm_delete: None,
            ..p
        }),
        ProfileMsg::ConfirmDelete => match state.profiles.confirm_delete.clone() {
            Some(name) => {
                let (state, _) = with(state, |p| ProfileState {
                    confirm_delete: None,
                    ..p
                });
                (state, vec![Effect::DeleteProfile(name)])
            }
            None => (state, Vec::new()),
        },
        ProfileMsg::Changed(Err(e)) => failed(state, format!("The profile change failed: {e}")),
        ProfileMsg::Changed(Ok((list, app))) => changed(state, list, app),
    }
}

/// The profile in use changed (or only the list did): close the vault when the settings changed and
/// reopen the profile's own last vault.
fn changed(state: State, list: Profiles, app: AppSettings) -> (State, Vec<Effect>) {
    let switched = list.current != state.profiles.list.current;
    let profiles = ProfileState {
        list,
        draft: String::new(),
        confirm_delete: None,
    };
    if !switched {
        return (State { profiles, ..state }, Vec::new());
    }
    let state = State {
        profiles,
        vault: None,
        screen: Screen::Welcome,
        polling: false,
        home: None,
        editor: None,
        modal: None,
        ..state
    };
    crate::update::started(state, app, None)
}
