//! Performing effects. No egui here: the runtime thread and the tests both call [`execute`].
//! The use cases themselves live in `studyfrog-service`; this only turns results into messages.

use std::path::PathBuf;

use studyfrog_core::settings::AppSettings;
use studyfrog_service::create::{IdSource, create_card, create_stack, fresh_id_source};
use studyfrog_service::session::{ServiceError, VaultInfo, VaultSession, open_vault, poll_vault};
use studyfrog_store::app_config::{read_app_settings, write_app_settings};
use studyfrog_store::overview::list_items;

use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::Selection;

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
        Effect::LoadItems(selection) => load_items(w, &selection),
        Effect::CreateStack(title) => create(w, now_ns, |s, ids| create_stack(s, &title, ids)),
        Effect::CreateCard {
            folder,
            title,
            front,
            back,
        } => create(w, now_ns, |s, ids| {
            create_card(s, &folder, &title, &front, &back, ids)
        }),
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

fn load_items(w: &Worker, selection: &Selection) -> Msg {
    let folder = match selection {
        Selection::Stack(f) => Some(f.as_str()),
        Selection::Inbox | Selection::Nothing => None,
    };
    match &w.session {
        Some(s) => Msg::ItemsLoaded(list_items(&s.conn, folder).map_err(|e| e.to_string())),
        None => Msg::ItemsLoaded(Ok(Vec::new())),
    }
}

/// Run a creating use case with fresh ids and report the refreshed vault.
fn create(
    w: &mut Worker,
    now_ns: i64,
    f: impl FnOnce(&mut VaultSession, IdSource) -> Result<VaultInfo, ServiceError>,
) -> Msg {
    let Some(session) = &mut w.session else {
        return Msg::Created(Err("no vault is open".to_owned()));
    };
    let result = fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
        .and_then(|ids| f(session, ids));
    Msg::Created(result.map_err(|e| e.to_string()))
}
