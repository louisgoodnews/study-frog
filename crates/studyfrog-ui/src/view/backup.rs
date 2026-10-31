//! The backup and restore section of the settings screen.

use eframe::egui::{self, RichText};
use studyfrog_md::rfc3339::format_rfc3339;

use crate::backup_ui::{BackupMsg, Chosen};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: BackupMsg) {
    out.push(Msg::Backup(m));
}

/// The section.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let b = &state.backup;
    let weak = palette_of(ui).text_weak;
    kit::section(ui, "Backup and restore");
    ui.label(
        RichText::new("A backup is one .tar file with your notes, pictures, review history, board, calendar and settings.")
            .color(weak),
    );
    let mut keys = b.include_keys;
    if ui
        .checkbox(
            &mut keys,
            "Include API keys (they are not encrypted in the backup)",
        )
        .changed()
    {
        send(out, BackupMsg::IncludeKeys(keys));
    }
    ui.horizontal(|ui| {
        let start = ui.add_enabled(
            !b.busy && state.vault.is_some(),
            egui::Button::new("Back up now…"),
        );
        if start.clicked() {
            send(out, BackupMsg::Start);
        }
        if ui
            .add_enabled(!b.busy, egui::Button::new("Restore from a backup…"))
            .clicked()
        {
            send(out, BackupMsg::Choose);
        }
        if b.busy {
            ui.spinner();
        }
    });
    if let Some(last) = &b.last {
        ui.label(last);
    }
    if let Some(c) = &b.chosen {
        chosen(state, c, ui, out);
    }
}

pub(super) fn chosen(state: &State, c: &Chosen, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let m = &c.manifest;
    let when = format_rfc3339(studyfrog_core::time::UnixMs(m.created_ms))
        .map_or(String::new(), |t| t[..10].to_owned());
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(format!("Backup of {} from {when}", m.vault_name)).strong());
        ui.label(format!(
            "{} files, {} KB, made by StudyFrog {}",
            m.files,
            m.bytes / 1024,
            m.app_version
        ));
        let yes = |on: bool| if on { "yes" } else { "no" };
        ui.label(format!(
            "Settings inside: {}. API keys inside: {}.",
            yes(m.has_settings),
            yes(m.has_keys)
        ));
        ui.label("It is restored into a new or empty folder; nothing existing is overwritten.");
        if m.has_settings {
            let mut on = state.backup.restore_settings;
            if ui
                .checkbox(
                    &mut on,
                    "Also replace my settings with the ones in the backup",
                )
                .changed()
            {
                send(out, BackupMsg::RestoreSettings(on));
            }
        }
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Restore into a new folder…").clicked() {
                send(out, BackupMsg::Restore);
            }
            if ui.button("Cancel restore").clicked() {
                send(out, BackupMsg::Dismiss);
            }
        });
    });
}
