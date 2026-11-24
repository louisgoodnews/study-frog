//! Performing effects. No egui here: the runtime thread and the tests both call [`execute`].
//! The use cases themselves live in `studyfrog-service`; this only turns results into messages.
use crate::effect::EffectAssistant;
use crate::msg::MsgRehearsal;
use crate::msg::MsgVault;

use std::path::PathBuf;

#[rustfmt::skip]
use studyfrog_core::{plan::RunPlan, run::RunMode, settings::AppSettings, time::UnixMs, uid::Uid};
use studyfrog_rehearsal::queue::candidate::budget_unlimited;
use studyfrog_service::queue::{NewRun, preview_queue};
use studyfrog_service::session::{VaultSession, current_settings, open_vault};
use studyfrog_service::timezone::local_day_window;
use studyfrog_slm::models::Opened;

use studyfrog_ext::types::Registry;
pub mod console;

use crate::host::{FilePicker, FolderPicker};
use studyfrog_store::app_config::write_app_settings;
use studyfrog_store::overview::search_items;

use crate::chrome::{ChromeMsg, MAX_RESULTS};
use crate::effect::Effect;
use crate::msg::Msg;
use crate::{exec_agent, exec_doc, exec_pdf, exec_plan, exec_run};

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
    /// Fetches the local Ollama status (GET, local address only).
    pub get: Option<studyfrog_agent::provider::HttpGet>,
    /// Where the agent logs its calls (never their content).
    pub state_dir: Option<PathBuf>,
    /// The PDF read last (vault-relative path and bytes), kept for the page requests that follow.
    pub pdf_bytes: Option<(String, studyfrog_pdf::PdfBytes)>,
    /// Shows a folder in the system's file manager, if the host can.
    pub opener: Option<crate::host::Opener>,
}

pub use crate::exec_worker::{worker_new, worker_with_host};

mod areas;
mod classroom;
mod mock;

/// Perform one effect and report what happened. `now_ns` is the current Unix time in nanoseconds.
pub fn execute(w: &mut Worker, effect: Effect, now_ns: i64) -> Vec<Msg> {
    let msg = match effect {
        Effect::Vault(e) => return areas::vault(w, e, now_ns),
        Effect::Backup(e) => return areas::backup(w, e, now_ns),
        Effect::Import(e) => return areas::import(w, e, now_ns),
        Effect::Browse(e) => return areas::browse(w, e, now_ns),
        Effect::Editor(e) => return areas::editor(w, e, now_ns),
        Effect::Rehearsal(e) => return areas::rehearsal(w, e, now_ns),
        Effect::Planner(e) => return areas::planner(w, e, now_ns),
        Effect::Reports(e) => return areas::reports(w, e, now_ns),
        Effect::Library(e) => crate::exec_library::run(w, e, now_ns),
        Effect::StackMark(e) => return crate::exec_mark::execute(w, e, now_ns),
        Effect::Folders(e) => return crate::exec_folders::execute(w, e, now_ns),
        Effect::Notifications(op) => return crate::exec_items::notifications(w, op, now_ns),
        Effect::Answer(crate::run_state::AnswerEffect::Record { run, item, scored }) => {
            crate::exec_run::record_case(w, run, item, &scored, now_ns)
        }
        Effect::MockExam(e) => mock::run(w, e, now_ns),
        Effect::Learn(e) => crate::exec_learn::run(w, e, now_ns),
        Effect::Console(e) => console::run(w, e, now_ns),
        Effect::LoadHomework => return crate::exec_homework::load(w, now_ns),
        Effect::Homework(e) => crate::exec_homework::execute(w, e, now_ns),
        Effect::Reviews(e) => exec_plan::reviews_effect(w, e, now_ns),
        Effect::Help(e) => crate::exec_help::run(w, e),
        Effect::Prompts(e) => crate::exec_prompts::run(w, e),
        Effect::Diary(e) => return crate::exec_diary::execute(w, e, now_ns),
        Effect::Agent(e) => crate::exec_mcp::run(w, e, now_ns),
        Effect::Classroom(e) => classroom::run(w, e, now_ns),
        e @ (Effect::Assistant(EffectAssistant::LoadAgent)
        | Effect::Assistant(EffectAssistant::SaveConnection { .. })
        | Effect::Assistant(EffectAssistant::PrepareWiki { .. })
        | Effect::Assistant(EffectAssistant::AgentChat { .. })
        | Effect::Assistant(EffectAssistant::ApplyChanges(_))
        | Effect::Assistant(EffectAssistant::PrepareExam { .. })
        | Effect::Assistant(EffectAssistant::RecordExam(_))) => exec_agent::run(w, e, now_ns),
        Effect::Pdf(e) => exec_pdf::run(w, e, now_ns),
        Effect::RehearsalEdit(crate::rehearsal_edit::RehearsalEditEffect::Load(uid)) => {
            exec_doc::load_rehearsal_doc_msg(w, uid)
        }
        Effect::RehearsalEdit(crate::rehearsal_edit::RehearsalEditEffect::Save(
            uid,
            path,
            doc,
            hash,
        )) => exec_doc::save_rehearsal_doc_msg(w, uid, &path, &doc, hash, now_ns),
        Effect::RehearsalEdit(crate::rehearsal_edit::RehearsalEditEffect::AppendComment(
            uid,
            comment,
        )) => exec_doc::append_rehearsal_comment_msg(w, uid, comment, now_ns),
        Effect::Exam(e) => crate::exec_exam::execute(w, e, now_ns),
        Effect::Answer(crate::run_state::AnswerEffect::Score {
            run,
            item,
            answer,
            words_only,
        }) => exec_run::score(w, (run, item), &answer, words_only, now_ns),
    };
    vec![msg]
}

fn save_settings(w: &Worker, settings: &AppSettings) -> Msg {
    let result = match &w.config_dir {
        Some(dir) => write_app_settings(dir, settings).map_err(|e| e.to_string()),
        None => Ok(()),
    };
    Msg::Vault(MsgVault::AppSettingsSaved(result))
}

fn open(w: &mut Worker, path: &str) -> Msg {
    match open_vault(path) {
        Ok((session, info)) => {
            w.session = Some(session);
            Msg::Vault(MsgVault::VaultOpened(Ok(info)))
        }
        Err(e) => Msg::Vault(MsgVault::VaultOpened(Err(e.to_string()))),
    }
}

/// Count the items a run would contain. Daily limits count from the start of the local day.
fn preview(w: &Worker, plan: RunPlan, mode: RunMode, now_ns: i64) -> Msg {
    let Some(session) = &w.session else {
        return Msg::Rehearsal(MsgRehearsal::PreviewReady(Err(
            "no vault is open".to_owned()
        )));
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
    Msg::Rehearsal(MsgRehearsal::PreviewReady(
        preview_queue(&session.conn, req, &current_settings(session)).map_err(|e| e.to_string()),
    ))
}

/// Find items by title for the global search.
fn search(w: &Worker, query: String) -> Msg {
    let result = match &w.session {
        Some(s) => search_items(&s.conn, &query, MAX_RESULTS).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    };
    Msg::Chrome(ChromeMsg::SearchResults(query, result))
}
