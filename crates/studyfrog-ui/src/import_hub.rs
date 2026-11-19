//! Universal Import Hub state, messages, format detection, and target resolution. Pure.

use crate::effect::Effect;
use crate::import_ui::ImportOutcome;
use crate::library_bulk::LibraryEffect;
use crate::modal::Modal;
use crate::state::{Notice, NoticeKind, Selection, State, VaultInfo};
pub use crate::transfer_types::{HubJob, HubState, HubTarget, ImportDestination};
use crate::transfer_types::{Tab, TransferJob, TransferState};
use crate::transfer_ui::TransferMsg;
pub use crate::view::modal_transfer_import::{
    DetectedFormat, detect_format, format_label, is_pdf_format,
};

/// Messages specific to the Universal Import Hub.
#[derive(Debug, Clone, PartialEq)]
pub enum HubMsg {
    /// Switch between Existing stack, New stack, and Unstacked vault notes.
    SetDest(ImportDestination),
    /// Edit the new stack name field.
    EditNewStack(String),
    /// A file or folder path was dropped onto the drop zone.
    Dropped(String),
    /// A dropped item could not be read as a path.
    DropError(String),
    /// Commit a PDF import into the chosen existing or new stack.
    ImportPdf,
    /// PDF import completed.
    PdfImported(Result<(String, VaultInfo), String>),
    /// Expand an extension card type and add cards into the chosen stack.
    CreateCardType,
    /// Extension card-type expansion completed.
    CardTypeCreated(Result<ImportOutcome, String>),
}

fn job(j: HubJob) -> Vec<Effect> {
    vec![Effect::Library(LibraryEffect::Transfer(TransferJob::Hub(
        j,
    )))]
}

/// Current [`HubTarget`] implied by `TransferState`.
pub fn current_target(t: &TransferState) -> HubTarget {
    match t.hub.dest_mode {
        ImportDestination::ExistingStack => HubTarget::Existing(t.scope.clone()),
        ImportDestination::NewStack => HubTarget::NewStack(t.hub.new_stack_name.trim().to_owned()),
        ImportDestination::UnstackedNotes => HubTarget::UnstackedNotes,
    }
}

/// Open the unified Import Hub with context-aware destination defaults.
pub fn open(state: State) -> (State, Vec<Effect>) {
    let (scope, dest) = match &state.browser.selection {
        Selection::Stack(folder) => (Some(folder.clone()), ImportDestination::ExistingStack),
        Selection::Inbox => (None, ImportDestination::UnstackedNotes),
        Selection::Nothing => (None, ImportDestination::ExistingStack),
    };
    let (mut state, fx) = crate::transfer_ui::update(state, TransferMsg::Open(Tab::Import, scope));
    state.transfer.hub.dest_mode = dest;
    (state, fx)
}

/// Initialize Hub defaults whenever `TransferMsg::Open` runs.
pub fn init_on_open(state: &mut State, tab: Tab) {
    state.transfer.hub = HubState::default();
    if tab == Tab::Import
        && state.transfer.scope.is_none()
        && matches!(state.browser.selection, Selection::Inbox)
    {
        state.transfer.hub.dest_mode = ImportDestination::UnstackedNotes;
    }
}

fn dispatch(state: &mut State, commit: bool) -> Vec<Effect> {
    let t = &mut state.transfer;
    if t.hub.dest_mode == ImportDestination::NewStack && t.hub.new_stack_name.trim().is_empty() {
        t.status = Some("Enter a name for the new stack first.".to_owned());
        return Vec::new();
    }
    t.busy = true;
    if !commit {
        t.status = None;
    }
    let (path, csv) = (t.path.trim().to_owned(), t.csv.clone());
    if t.hub.dest_mode == ImportDestination::ExistingStack {
        let j = if commit {
            TransferJob::Import {
                path,
                target: t.scope.clone(),
                policy: t.policy,
                with_schedule: t.with_schedule,
                csv,
            }
        } else {
            TransferJob::Preview(path, t.scope.clone(), csv)
        };
        return vec![Effect::Library(LibraryEffect::Transfer(j))];
    }
    let target = current_target(t);
    job(if commit {
        HubJob::ImportHub(path, target, t.policy, t.with_schedule, csv)
    } else {
        HubJob::PreviewHub(path, target, csv)
    })
}

/// Build the preview effect for the current hub state.
pub fn preview_effects(state: &mut State) -> Vec<Effect> {
    dispatch(state, false)
}

/// Build the import commit effect for the current hub state.
pub fn import_effects(state: &mut State) -> Vec<Effect> {
    dispatch(state, true)
}

fn valid_stack_target(t: &TransferState) -> bool {
    match t.hub.dest_mode {
        ImportDestination::ExistingStack => t.scope.is_some(),
        ImportDestination::NewStack => !t.hub.new_stack_name.trim().is_empty(),
        ImportDestination::UnstackedNotes => false,
    }
}

fn fail(mut state: State, msg: &str) -> (State, Vec<Effect>) {
    state.transfer.status = Some(msg.to_owned());
    (state, Vec::new())
}

fn import_pdf(mut state: State) -> (State, Vec<Effect>) {
    if state.transfer.path.trim().is_empty() {
        return fail(state, "Choose a PDF file first.");
    }
    if !valid_stack_target(&state.transfer) {
        return fail(
            state,
            "Choose an existing stack or enter a new stack name for the PDF.",
        );
    }
    (state.transfer.busy, state.transfer.status) = (true, None);
    let (path, target) = (
        state.transfer.path.trim().to_owned(),
        current_target(&state.transfer),
    );
    (state, job(HubJob::ImportPdf(path, target)))
}

fn create_card_type(mut state: State) -> (State, Vec<Effect>) {
    let Some(type_id) = state.browser.card_type.clone() else {
        return fail(state, "Choose a card type first.");
    };
    let source = state.browser.draft.front.clone();
    if source.trim().is_empty() {
        return fail(state, "Write the text to make cards from first.");
    }
    if !valid_stack_target(&state.transfer) {
        return fail(state, "Choose an existing or new stack for card types.");
    }
    (state.transfer.busy, state.transfer.status) = (true, None);
    let target = current_target(&state.transfer);
    (state, job(HubJob::CreateCardType(target, type_id, source)))
}

fn pdf_imported(
    mut state: State,
    res: Result<(String, VaultInfo), String>,
) -> (State, Vec<Effect>) {
    state.transfer.busy = false;
    let (kind, text) = match &res {
        Ok((rel, _)) => (NoticeKind::Info, format!("Added PDF {rel}.")),
        Err(e) => (NoticeKind::Error, format!("Could not add the PDF: {e}")),
    };
    if let Ok((_, info)) = res {
        (state.vault, state.library.stale) = (Some(info), true);
        state.transfer.path.clear();
    }
    state.transfer.status = Some(text.clone());
    state.notice = Some(Notice { kind, text });
    let fx = (kind == NoticeKind::Info)
        .then(crate::pdf_library_ui::load)
        .into_iter()
        .collect();
    (state, fx)
}

/// Handle a [`HubMsg`].
pub fn update(mut state: State, msg: HubMsg) -> (State, Vec<Effect>) {
    match msg {
        HubMsg::SetDest(mode) => {
            let t = &mut state.transfer;
            (t.hub.dest_mode, t.preview, t.status) = (mode, None, None);
            (state, Vec::new())
        }
        HubMsg::EditNewStack(name) => {
            let t = &mut state.transfer;
            (t.hub.new_stack_name, t.preview, t.status) = (name, None, None);
            (state, Vec::new())
        }
        HubMsg::Dropped(path) => {
            let t = &mut state.transfer;
            (t.path, t.hub.is_folder, t.preview, t.csv, t.status) =
                (path, false, None, Default::default(), None);
            let fmt = detect_format(&t.path, false, &state.extensions.importers);
            let fx = if is_pdf_format(fmt.as_ref()) {
                Vec::new()
            } else {
                preview_effects(&mut state)
            };
            (state, fx)
        }
        HubMsg::DropError(e) => fail(state, &e),
        HubMsg::ImportPdf => import_pdf(state),
        HubMsg::PdfImported(res) => pdf_imported(state, res),
        HubMsg::CreateCardType => create_card_type(state),
        HubMsg::CardTypeCreated(res) => {
            state.transfer.busy = false;
            let (mut s, fx) = crate::import_ui::update(state, crate::msg::Msg::Imported(res));
            if s.notice
                .as_ref()
                .is_some_and(|n| n.kind == NoticeKind::Info)
                && s.modal == Some(Modal::Transfer)
            {
                s.modal = None;
            }
            (s, fx)
        }
    }
}
