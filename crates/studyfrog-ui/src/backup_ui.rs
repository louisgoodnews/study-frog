//! Backing up and restoring on the settings screen. Pure.

use studyfrog_config::backup::Manifest;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// A backup picked for restoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// The archive.
    pub path: String,
    /// What it holds.
    pub manifest: Manifest,
}

/// The backup section's state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackupState {
    /// Put the API keys in the next backup.
    pub include_keys: bool,
    /// A backup or restore is running.
    pub busy: bool,
    /// The backup waiting to be restored.
    pub chosen: Option<Chosen>,
    /// Replace the app settings with those of the backup.
    pub restore_settings: bool,
    /// What the last backup did, in words.
    pub last: Option<String>,
}

/// Messages of the backup section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupMsg {
    /// Include the keys or not.
    IncludeKeys(bool),
    /// Make a backup (asks where to put it).
    Start,
    /// The backup finished: what it did, or `None` when the user cancelled.
    Done(Result<Option<String>, String>),
    /// Choose a backup to restore.
    Choose,
    /// A backup was chosen (`None`: cancelled) and read.
    Chosen(Result<Option<Chosen>, String>),
    /// Restore the settings too or not.
    RestoreSettings(bool),
    /// Restore the chosen backup (asks for an empty folder).
    Restore,
    /// The restore finished: the new vault folder, or `None` when the user cancelled.
    Restored(Result<Option<String>, String>),
    /// Forget the chosen backup.
    Dismiss,
}

fn failed(state: State, what: &str, e: String) -> (State, Vec<Effect>) {
    let backup = BackupState {
        busy: false,
        ..state.backup.clone()
    };
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text: format!("{what}: {e}"),
    });
    (
        State {
            backup,
            notice,
            ..state
        },
        Vec::new(),
    )
}

fn with(state: State, f: impl FnOnce(BackupState) -> BackupState) -> (State, Vec<Effect>) {
    let backup = f(state.backup.clone());
    (State { backup, ..state }, Vec::new())
}

/// Handle the messages of the backup section.
pub fn update(state: State, msg: BackupMsg) -> (State, Vec<Effect>) {
    match msg {
        BackupMsg::IncludeKeys(on) => with(state, |b| BackupState {
            include_keys: on,
            ..b
        }),
        BackupMsg::RestoreSettings(on) => with(state, |b| BackupState {
            restore_settings: on,
            ..b
        }),
        BackupMsg::Dismiss => with(state, |b| BackupState { chosen: None, ..b }),
        BackupMsg::Start if state.backup.busy || state.vault.is_none() => (state, Vec::new()),
        BackupMsg::Start => {
            let keys = state.backup.include_keys;
            let (state, _) = with(state, |b| BackupState { busy: true, ..b });
            (state, vec![Effect::Backup { include_keys: keys }])
        }
        BackupMsg::Done(Ok(last)) => with(state, |b| BackupState {
            busy: false,
            last: last.or(b.last),
            ..b
        }),
        BackupMsg::Done(Err(e)) => failed(state, "The backup failed", e),
        BackupMsg::Choose => (state, vec![Effect::ChooseBackup]),
        BackupMsg::Chosen(Ok(chosen)) => with(state, |b| BackupState {
            chosen: chosen.or(b.chosen),
            ..b
        }),
        BackupMsg::Chosen(Err(e)) => failed(state, "That is not a usable backup", e),
        BackupMsg::Restore => match (&state.backup.chosen, state.backup.busy) {
            (Some(c), false) => {
                let effect = Effect::Restore {
                    archive: c.path.clone(),
                    with_settings: state.backup.restore_settings,
                };
                let (state, _) = with(state, |b| BackupState { busy: true, ..b });
                (state, vec![effect])
            }
            _ => (state, Vec::new()),
        },
        BackupMsg::Restored(Ok(None)) => with(state, |b| BackupState { busy: false, ..b }),
        BackupMsg::Restored(Ok(Some(folder))) => {
            let (state, _) = with(state, |b| BackupState {
                busy: false,
                chosen: None,
                last: Some(format!("Restored into {folder}")),
                ..b
            });
            (state, vec![Effect::OpenVault(folder)])
        }
        BackupMsg::Restored(Err(e)) => failed(state, "The restore failed", e),
    }
}
