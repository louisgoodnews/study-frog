//! The suggestions of the import dialog: what was found on this computer to import.

use eframe::egui;
use studyfrog_service::xfer_discover::{Found, FoundKind};

use crate::msg::Msg;
use crate::state::State;
use crate::transfer_ui::TransferMsg;
use crate::view::kit;

fn kind_name(kind: FoundKind) -> &'static str {
    match kind {
        FoundKind::ObsidianVault => "Obsidian vault",
        FoundKind::AnkiPackage => "Anki package",
        FoundKind::AnkiProfile => "Anki profile",
    }
}

fn one(f: &Found, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        let what = kind_name(f.kind);
        if f.kind == FoundKind::AnkiProfile {
            ui.label(format!("{} ({what}): {}", f.name, f.detail));
            return;
        }
        if ui
            .button("Use")
            .on_hover_text(f.path.as_str())
            .labelled_by(ui.label(format!("{} · {what} · {}", f.name, f.detail)).id)
            .clicked()
        {
            out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(
                TransferMsg::Pick(f.path.clone()),
            )));
        }
    });
}

/// What was found, or that the search is running or found nothing.
pub fn found(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    kit::section(ui, "Found on this computer");
    match &t.found {
        None => {
            ui.weak("Looking for Obsidian vaults and Anki files…");
        }
        Some(list) if list.is_empty() => {
            ui.weak("No Obsidian vault or Anki package found in the usual places. Choose a file or folder below.");
        }
        Some(list) => {
            ui.weak("Nothing is read until you choose one; you see a preview first.");
            for f in list {
                one(f, ui, out);
            }
        }
    }
    ui.separator();
}
