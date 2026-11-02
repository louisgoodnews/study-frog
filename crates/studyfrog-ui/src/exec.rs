//! Performing effects. No egui here: the runtime thread and the tests both call [`execute`].
//! The use cases themselves live in `studyfrog-service`; this only turns results into messages.

use std::path::PathBuf;

use studyfrog_core::plan::RunPlan;
use studyfrog_core::run::RunMode;
use studyfrog_core::settings::AppSettings;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_service::queue::{NewRun, preview_queue};
use studyfrog_service::session::{VaultSession, current_settings, open_vault};
use studyfrog_service::timezone::local_day_window;
use studyfrog_service::track::create_card_tracked;
use studyfrog_slm::models::Opened;

use studyfrog_ext::types::Registry;

use crate::host::{FilePicker, FolderPicker, Host};
use studyfrog_store::app_config::write_app_settings;
use studyfrog_store::overview::search_items;

use crate::chrome::{ChromeMsg, MAX_RESULTS};
use crate::effect::Effect;
use crate::msg::Msg;
use crate::{
    exec_agent, exec_backup, exec_doc, exec_import, exec_pdf, exec_plan, exec_profile, exec_run,
};

/// Executor state: where per-user settings live and which vault is open.
pub struct Worker {
    /// Directory holding `app.toml`; `None` when no home directory is known.
    pub config_dir: Option<PathBuf>,
    /// The folder above all profiles (`None` when no home directory is known).
    pub root_config: Option<PathBuf>,
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
    /// Sends the AI agent's requests.
    pub transport: Option<studyfrog_agent::provider::Transport>,
    /// Where the agent logs its calls (never their content).
    pub state_dir: Option<PathBuf>,
    /// The PDF read last (vault-relative path and bytes), kept for the page requests that follow.
    pub pdf_bytes: Option<(String, studyfrog_pdf::PdfBytes)>,
}

/// A worker with nothing open.
pub fn worker_new(config_dir: Option<PathBuf>) -> Worker {
    Worker {
        root_config: config_dir.clone(),
        config_dir,
        session: None,
        models_dir: None,
        slm: None,
        loader: None,
        folder_picker: None,
        file_picker: None,
        registry: Registry::default(),
        transport: None,
        state_dir: None,
        pdf_bytes: None,
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
        transport: host.transport,
        ..worker_new(config_dir)
    }
}

/// Perform one effect and report what happened. `now_ns` is the current Unix time in nanoseconds.
pub fn execute(w: &mut Worker, effect: Effect, now_ns: i64) -> Vec<Msg> {
    let msg = match effect {
        Effect::LoadAppSettings => return exec_profile::startup(w),
        Effect::CreateProfile(name) => exec_profile::create(w, &name),
        Effect::SwitchProfile(name) => exec_profile::switch(w, &name),
        Effect::DeleteProfile(name) => exec_profile::delete(w, &name),
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
        Effect::PollVault => exec_doc::poll(w, now_ns),
        Effect::CheckReminders => exec_plan::reminders(w, now_ns),
        Effect::SaveAppSettings(settings) => save_settings(w, &settings),
        Effect::LoadItems(selection) => crate::exec_items::items(w, &selection),
        Effect::RecordView(v) => return crate::exec_items::record_view(w, v, now_ns),
        Effect::LoadLibrary => crate::exec_items::library(w, now_ns),
        Effect::Library(e) => crate::exec_library::run(w, e, now_ns),
        Effect::Notifications(op) => return crate::exec_items::notifications(w, op, now_ns),
        Effect::LoadStatusCounts(selection) => crate::exec_items::counts(w, &selection),
        Effect::PreviewQueue { plan, mode } => preview(w, plan, mode, now_ns),
        Effect::StartRun { plan, mode } => exec_run::start(w, plan, mode, now_ns),
        Effect::LoadGraph => exec_doc::load_graph_msg(w),
        Effect::LoadSessions => exec_plan::load_sessions(w, now_ns),
        Effect::Reviews(e) => exec_plan::reviews_effect(w, e, now_ns),
        Effect::SaveSession(s) => exec_plan::save(w, s, now_ns),
        Effect::DeleteSession(id) => exec_plan::delete(w, id),
        Effect::CancelOccurrence(id, date) => exec_plan::skip(w, id, date),
        Effect::SearchItems(query) => search(w, query),
        Effect::LoadHome => exec_run::home(w, now_ns),
        e @ (Effect::LoadAgent
        | Effect::SaveConnection { .. }
        | Effect::RemoveConnection(_)
        | Effect::AllowConnection(..)
        | Effect::PrepareWiki { .. }
        | Effect::AgentChat { .. }
        | Effect::ApplyChanges(_)
        | Effect::PrepareExam { .. }
        | Effect::RecordExam(_)) => exec_agent::run(w, e, now_ns),
        Effect::Backup { include_keys } => exec_backup::backup(w, include_keys, now_ns),
        Effect::ChooseBackup => exec_backup::choose(w),
        Effect::Restore {
            archive,
            with_settings,
        } => exec_backup::restore(w, &archive, with_settings),
        Effect::AddImage(target) => exec_import::add_image(w, target),
        Effect::LoadSubjects => exec_doc::subjects(w, None),
        Effect::SaveSubjects(book) => exec_doc::subjects(w, Some(book)),
        Effect::LoadBacklinks(uid, title) => exec_doc::backlinks(w, uid, &title),
        Effect::CreateItem {
            kind,
            title,
            text,
            answer,
            tags,
            parent,
            due,
        } => exec_import::create_item(
            w,
            kind,
            [&title, &text, &answer],
            (&tags, parent.as_deref(), due.as_ref()),
            now_ns,
        ),
        Effect::Pdf(e) => exec_pdf::run(w, e, now_ns),
        Effect::BuildReports => exec_run::reports(w, now_ns),
        Effect::Explore { scope, period } => exec_run::explorer(w, (scope, period), now_ns),
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
        Effect::CreateCard {
            folder,
            title,
            front,
            back,
        } => exec_import::create(w, now_ns, |s, i| {
            create_card_tracked(s, (&folder, &title, &front, &back), i)
        }),
    };
    vec![msg]
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
