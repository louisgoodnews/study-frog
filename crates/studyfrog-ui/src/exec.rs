//! Performing effects. No egui here: the runtime thread and the tests both call [`execute`].
//! The use cases themselves live in `studyfrog-service`; this only turns results into messages.

use std::path::PathBuf;

use studyfrog_core::plan::RunPlan;
use studyfrog_core::run::RunMode;
use studyfrog_core::settings::AppSettings;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_service::create::{IdSource, create_card, create_stack, fresh_id_source};
use studyfrog_service::queue::{NewRun, preview_queue};
use studyfrog_service::session::{
    ServiceError, VaultInfo, VaultSession, current_settings, open_vault, poll_vault,
};
use studyfrog_service::timezone::local_day_window;
use studyfrog_slm::models::Opened;

use studyfrog_ext::types::Registry;

use crate::host::{FilePicker, FolderPicker, Host};
use studyfrog_store::app_config::{read_app_settings, write_app_settings};
use studyfrog_store::overview::{list_items, search_items};

use crate::chrome::{ChromeMsg, MAX_RESULTS};
use crate::effect::Effect;
use crate::msg::Msg;
use crate::state::Selection;
use crate::{exec_doc, exec_import, exec_plan, exec_run};

/// Executor state: where per-user settings live and which vault is open.
pub struct Worker {
    /// Directory holding `app.toml`; `None` when no home directory is known.
    pub config_dir: Option<PathBuf>,
    /// The open vault.
    pub session: Option<VaultSession>,
    /// Where language models are installed; `None` means word matching only.
    pub models_dir: Option<PathBuf>,
    /// The answer-comparison runtime, opened on first use, and why it is not the semantic one.
    pub slm: Option<Opened>,
    /// How to open the semantic runtime; `None` for a build without one.
    pub loader: Option<studyfrog_slm::models::Loader>,
    /// The native folder dialog, if the host has one.
    pub folder_picker: Option<FolderPicker>,
    /// The native file dialog, if the host has one.
    pub file_picker: Option<FilePicker>,
    /// The importers and card types extensions registered.
    pub registry: Registry,
}

/// A worker with nothing open.
pub fn worker_new(config_dir: Option<PathBuf>) -> Worker {
    Worker {
        config_dir,
        session: None,
        models_dir: None,
        slm: None,
        loader: None,
        folder_picker: None,
        file_picker: None,
        registry: Registry::default(),
    }
}

/// A worker that looks for language models in `models_dir` and uses what the host provides.
pub fn worker_with_host(
    config_dir: Option<PathBuf>,
    models_dir: Option<PathBuf>,
    host: Host,
) -> Worker {
    Worker {
        models_dir,
        loader: host.model_loader,
        folder_picker: host.folder_picker,
        file_picker: host.file_picker,
        registry: host.registry,
        ..worker_new(config_dir)
    }
}

/// Perform one effect and report what happened. `now_ns` is the current Unix time in nanoseconds.
pub fn execute(w: &mut Worker, effect: Effect, now_ns: i64) -> Vec<Msg> {
    let msg = match effect {
        Effect::LoadAppSettings => load_settings(w),
        Effect::PickImportFile => exec_import::pick_file(w),
        Effect::ImportFile { folder, path } => exec_import::import_file(w, &folder, &path, now_ns),
        Effect::CreateFromType {
            folder,
            type_id,
            source,
        } => exec_import::from_card_type(w, &folder, &type_id, &source, now_ns),
        Effect::PickVaultFolder => Msg::VaultFolderPicked(
            w.folder_picker
                .and_then(|pick| pick())
                .map(|p| p.display().to_string()),
        ),
        Effect::OpenVault(path) => open(w, &path),
        Effect::PollVault => poll(w, now_ns),
        Effect::SaveAppSettings(settings) => save_settings(w, &settings),
        Effect::LoadItems(selection) => load_items(w, &selection),
        Effect::PreviewQueue { plan, mode } => preview(w, plan, mode, now_ns),
        Effect::StartRun { plan, mode } => exec_run::start(w, plan, mode, now_ns),
        Effect::LoadGraph => exec_doc::load_graph_msg(w),
        Effect::LoadSessions => exec_plan::load_sessions(w, now_ns),
        Effect::SaveSession(s) => exec_plan::save(w, s, now_ns),
        Effect::DeleteSession(id) => exec_plan::delete(w, id),
        Effect::CancelOccurrence(id, date) => exec_plan::skip(w, id, date),
        Effect::SearchItems(query) => search(w, query),
        Effect::BuildReports => exec_run::reports(w, now_ns),
        Effect::CheckAchievements => exec_run::achievements(w, now_ns),
        Effect::LoadBoard => exec_plan::load_board(w, now_ns),
        Effect::BoardCommand(op) => exec_plan::board_command(w, &op, now_ns),
        Effect::LoadDocument(uid) => exec_doc::load_document_msg(w, uid),
        Effect::SaveDocument { path, doc, hash } => {
            exec_doc::save_document_msg(w, &path, &doc, hash, now_ns)
        }
        Effect::ResolveLink(target) => exec_doc::resolve_link_msg(w, target),
        Effect::Rerate { item, expected, to } => exec_run::rerate(w, item, expected, to),
        Effect::LoadItem(uid) => exec_run::load_item(w, uid),
        Effect::ScoreAnswer {
            run,
            item,
            answer,
            words_only,
        } => exec_run::score(w, (run, item), &answer, words_only, now_ns),
        Effect::RecordStep { ctx, out } => exec_run::record(w, &ctx, &out, now_ns),
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

/// Count the items a run would contain. Daily limits count from the start of the local day.
fn preview(w: &Worker, plan: RunPlan, mode: RunMode, now_ns: i64) -> Msg {
    let Some(session) = &w.session else {
        return Msg::PreviewReady(Err("no vault is open".to_owned()));
    };
    let now_ms = now_ns / 1_000_000;
    let day = local_day_window(UnixMs(now_ms));
    let req = NewRun {
        plan,
        mode,
        uid: Uid(0),
        now: UnixMs(now_ms),
        day,
        global_budget: budget_unlimited(),
    };
    Msg::PreviewReady(
        preview_queue(&session.conn, req, &current_settings(session)).map_err(|e| e.to_string()),
    )
}

/// Find items by title for the global search.
fn search(w: &Worker, query: String) -> Msg {
    let result = match &w.session {
        Some(s) => search_items(&s.conn, &query, MAX_RESULTS).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    };
    Msg::Chrome(ChromeMsg::SearchResults(query, result))
}
