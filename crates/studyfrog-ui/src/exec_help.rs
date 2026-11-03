//! Performing the help screen's effects: reading and writing the user's pages in the vault's `wiki/` folder.

use studyfrog_service::help_user::{delete_user_page, list_user_pages, save_user_page};

use crate::exec::Worker;
use crate::help_types::{HelpEffect, HelpMsg};
use crate::msg::Msg;

/// Perform one effect of the help screen and report what happened.
pub fn run(w: &Worker, effect: HelpEffect) -> Msg {
    let Some(root) = w.session.as_ref().map(|s| s.root.as_path()) else {
        return Msg::Help(HelpMsg::Loaded(Ok(Vec::new())));
    };
    let text = |e: studyfrog_service::session::ServiceError| e.to_string();
    let m = match effect {
        HelpEffect::Load => HelpMsg::Loaded(list_user_pages(root).map_err(text)),
        HelpEffect::Save { old, name, text: t } => {
            HelpMsg::Saved(save_user_page(root, old.as_deref(), &name, &t).map_err(text))
        }
        HelpEffect::Delete(name) => HelpMsg::Deleted(delete_user_page(root, &name).map_err(text)),
    };
    Msg::Help(m)
}
