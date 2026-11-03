//! Suggesting what to import: the dialog asks the worker to look for Obsidian vaults and Anki files when the
//! Import half opens, lists what was found and previews the one the user picks. Pure except `look`.

use studyfrog_service::xfer_discover::{Found, discover, places};

use crate::effect::Effect;
use crate::library_bulk::LibraryEffect;
use crate::library_ui::LibraryMsg;
use crate::modal::Modal;
use crate::msg::Msg;
use crate::state::State;
use crate::transfer_types::{Tab, TransferJob};
use crate::transfer_ui::TransferMsg;

/// Ask for the search once per opening of the dialog, when the Import half is showing.
pub fn want(state: &mut State) -> Vec<Effect> {
    let t = &mut state.transfer;
    let showing = state.modal == Some(Modal::Transfer) && t.tab == Tab::Import;
    if !showing || t.found.is_some() || t.looking {
        return Vec::new();
    }
    t.looking = true;
    vec![Effect::Library(LibraryEffect::Transfer(
        TransferJob::Discover,
    ))]
}

/// The search finished.
pub fn found(state: &mut State, list: Vec<Found>) {
    state.transfer.looking = false;
    state.transfer.found = Some(list);
}

/// The user chose a suggestion: it becomes the path and the preview is asked for.
pub fn pick(state: &mut State, path: String) -> Vec<Effect> {
    let t = &mut state.transfer;
    t.path = path.clone();
    t.preview = None;
    t.busy = true;
    t.status = None;
    let job = TransferJob::Preview(path, t.scope.clone());
    vec![Effect::Library(LibraryEffect::Transfer(job))]
}

/// Look in the user's usual places (the worker's thread; only names and counts are read).
pub fn look() -> Msg {
    let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    let msg = match var("HOME") {
        Some(home) => {
            let p = places(
                std::path::Path::new(&home),
                var("XDG_DATA_HOME").as_deref(),
                var("XDG_CONFIG_HOME").as_deref(),
            );
            TransferMsg::Found(discover(&p))
        }
        None => TransferMsg::Found(Vec::new()),
    };
    Msg::Library(LibraryMsg::Transfer(msg))
}
