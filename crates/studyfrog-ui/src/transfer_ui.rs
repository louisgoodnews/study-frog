//! The import and export dialog: state, messages and update. Pure; the worker reads and writes the files.

use studyfrog_service::xfer_plan::Policy;

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
use crate::modal::Modal;
use crate::state::{Notice, NoticeKind, State};
pub use crate::transfer_types::{Format, Imported, Preview, Tab, TransferJob, TransferState};

/// What happened in the dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferMsg {
    /// Open the dialog on a tab, for a stack or the library.
    Open(Tab, Option<String>),
    /// Show the other half.
    Tab(Tab),
    /// Choose the stack (or the whole library).
    Scope(Option<String>),
    /// Choose the export format.
    Format(Format),
    /// "Export…" was pressed: ask for the mode.
    AskMode,
    /// Choose the mode: advanced includes scheduling.
    Mode(bool),
    /// Back from the mode question.
    BackToExport,
    /// The destination folder was edited.
    Destination(String),
    /// The import path was edited.
    Path(String),
    /// Browse for a file (`false`) or a folder (`true`).
    Browse(bool, bool),
    /// The dialog for browsing answered.
    Browsed(bool, Option<String>),
    /// Write the export.
    Export,
    /// The export finished: the path written, or why not.
    Exported(Result<String, String>),
    /// Choose what to do with duplicates.
    Policy(Policy),
    /// Tick or untick the review history.
    WithSchedule(bool),
    /// Read the file and show the preview.
    Preview,
    /// The preview arrived.
    Previewed(Result<Box<Preview>, String>),
    /// Write the import.
    Import,
    /// The import finished.
    Imported(Result<Box<Imported>, String>),
}

fn job(effect: TransferJob) -> Vec<Effect> {
    vec![Effect::Library(LibraryEffect::Transfer(effect))]
}

fn say(state: &mut State, text: String, kind: NoticeKind) {
    state.transfer.status = Some(text.clone());
    state.notice = Some(Notice { kind, text });
}

/// Handle a message of the dialog.
pub fn update(state: State, msg: TransferMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let t = &mut state.transfer;
    match msg {
        TransferMsg::Open(tab, scope) => {
            *t = TransferState {
                tab,
                scope,
                destination: t.destination.clone(),
                ..TransferState::default()
            };
            state.modal = Some(Modal::Transfer);
        }
        TransferMsg::Tab(tab) => (t.tab, t.status) = (tab, None),
        TransferMsg::Scope(s) => (t.scope, t.preview) = (s, None),
        TransferMsg::Format(f) => t.format = f,
        TransferMsg::AskMode => t.choosing_mode = true,
        TransferMsg::BackToExport => t.choosing_mode = false,
        TransferMsg::Mode(advanced) => t.scheduling = advanced,
        TransferMsg::Destination(text) => t.destination = text,
        TransferMsg::Path(text) => (t.path, t.preview) = (text, None),
        TransferMsg::Browse(_, folder) => return (state, job(TransferJob::Browse(folder))),
        TransferMsg::Browsed(folder, Some(p)) => {
            let target = if folder && t.tab == Tab::Export {
                &mut t.destination
            } else {
                &mut t.path
            };
            *target = p;
            t.preview = None;
        }
        TransferMsg::Browsed(_, None) => {}
        TransferMsg::Export => {
            t.choosing_mode = false;
            t.busy = true;
            let (folder, scope, format, scheduling) = (
                t.destination.trim().to_owned(),
                t.scope.clone(),
                t.format,
                t.scheduling && t.format == Format::Json,
            );
            return (
                state,
                job(TransferJob::Export {
                    folder,
                    scope,
                    format,
                    scheduling,
                }),
            );
        }
        TransferMsg::Exported(result) => {
            t.busy = false;
            match result {
                Ok(path) => say(&mut state, format!("Exported to {path}."), NoticeKind::Info),
                Err(e) => say(
                    &mut state,
                    format!("Could not export: {e}"),
                    NoticeKind::Error,
                ),
            }
        }
        TransferMsg::Policy(p) => t.policy = p,
        TransferMsg::WithSchedule(on) => t.with_schedule = on,
        TransferMsg::Preview => {
            t.busy = true;
            t.status = None;
            let (path, scope) = (t.path.trim().to_owned(), t.scope.clone());
            return (state, job(TransferJob::Preview(path, scope)));
        }
        TransferMsg::Previewed(result) => {
            t.busy = false;
            match result {
                Ok(p) => (t.with_schedule, t.preview) = (t.with_schedule && p.scheduling, Some(p)),
                Err(e) => (t.preview, t.status) = (None, Some(e)),
            }
        }
        TransferMsg::Import => {
            t.busy = true;
            let (path, target, policy) = (t.path.trim().to_owned(), t.scope.clone(), t.policy);
            let with_schedule = t.with_schedule;
            return (
                state,
                job(TransferJob::Import {
                    path,
                    target,
                    policy,
                    with_schedule,
                }),
            );
        }
        TransferMsg::Imported(result) => {
            t.busy = false;
            match result {
                Ok(done) => {
                    let text = format!(
                        "Imported: {} created, {} replaced, {} left out.",
                        done.created, done.replaced, done.skipped
                    );
                    state.vault = Some(done.info);
                    state.library.stale = true;
                    state.transfer.preview = None;
                    say(&mut state, text, NoticeKind::Info);
                }
                Err(e) => say(
                    &mut state,
                    format!("Could not import: {e}"),
                    NoticeKind::Error,
                ),
            }
        }
    }
    (state, Vec::new())
}
