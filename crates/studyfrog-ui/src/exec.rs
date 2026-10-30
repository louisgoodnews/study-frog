//! Performing effects. No egui here: the runtime thread and the tests both call [`execute`].
//! The use cases themselves live in `studyfrog-service`; this only turns results into messages.

use std::path::PathBuf;

use studyfrog_core::settings::AppSettings;
use studyfrog_service::session::{VaultSession, open_vault, poll_vault};
use studyfrog_store::app_config::{read_app_settings, write_app_settings};

use crate::effect::Effect;
use crate::msg::Msg;

/// Executor state: where per-user settings live and which vault is open.
pub struct Worker {
    /// Directory holding `app.toml`; `None` when no home directory is known.
    pub config_dir: Option<PathBuf>,
    /// The open vault.
    pub session: Option<VaultSession>,
}

/// A worker with nothing open.
pub fn worker_new(config_dir: Option<PathBuf>) -> Worker {
    Worker {
        config_dir,
        session: None,
    }
}

/// Perform one effect and report what happened. `now_ns` is the current Unix time in nanoseconds.
pub fn execute(w: &mut Worker, effect: Effect, now_ns: i64) -> Vec<Msg> {
    let msg = match effect {
        Effect::LoadAppSettings => load_settings(w),
        Effect::OpenVault(path) => open(w, &path),
        Effect::PollVault => poll(w, now_ns),
        Effect::SaveAppSettings(settings) => save_settings(w, &settings),
    };
    vec![msg]
}

fn load_settings(w: &Worker) -> Msg {
    let Some(dir) = &w.config_dir else {
        return Msg::Started {
            app: AppSettings::default(),
            error: None,
        };
    };
    match read_app_settings(dir) {
        Ok(app) => Msg::Started { app, error: None },
        Err(e) => Msg::Started {
            app: AppSettings::default(),
            error: Some(e.to_string()),
        },
    }
}

fn save_settings(w: &Worker, settings: &AppSettings) -> Msg {
    let result = match &w.config_dir {
        Some(dir) => write_app_settings(dir, settings).map_err(|e| e.to_string()),
        None => Ok(()),
    };
    Msg::AppSettingsSaved(result)
}

fn open(w: &mut Worker, path: &str) -> Msg {
    match open_vault(path) {
        Ok((session, info)) => {
            w.session = Some(session);
            Msg::VaultOpened(Ok(info))
        }
        Err(e) => Msg::VaultOpened(Err(e.to_string())),
    }
}

fn poll(w: &mut Worker, now_ns: i64) -> Msg {
    match &mut w.session {
        Some(session) => Msg::VaultPolled(poll_vault(session, now_ns).map_err(|e| e.to_string())),
        None => Msg::VaultPolled(Ok(None)),
    }
}
