//! The import and export dialog: pick the format and the scope, see the preview, choose what to do with duplicates, see the result.

use eframe::egui::{self, ComboBox, RichText, vec2};

use crate::library_ui::LibraryMsg;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::transfer_ui::{Format, Tab, TransferMsg};
use crate::view::kit;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: TransferMsg) {
    out.push(Msg::Library(LibraryMsg::Transfer(m)));
}

pub(super) fn scope_menu(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>, label: &str) {
    let stacks: Vec<_> = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .collect();
    let (none, name) = if label == "Export" {
        ("Whole library", "Export what")
    } else {
        ("As the file says", "Import into")
    };
    let current = state
        .transfer
        .scope
        .as_ref()
        .and_then(|f| stacks.iter().find(|s| s.folder == *f))
        .map_or_else(|| none.to_owned(), |s| s.title.clone());
    ComboBox::from_label(name)
        .selected_text(current)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(state.transfer.scope.is_none(), none)
                .clicked()
            {
                send(out, TransferMsg::Scope(None));
            }
            for s in &stacks {
                if ui
                    .selectable_label(state.transfer.scope.as_deref() == Some(&s.folder), &s.title)
                    .clicked()
                {
                    send(out, TransferMsg::Scope(Some(s.folder.clone())));
                }
            }
        });
}

fn mode_question(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    ui.add_space(8.0);
    kit::card(ui, |ui| {
        ui.label(
            RichText::new("Export mode")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.add_space(6.0);
        if ui
            .radio(!t.scheduling, "Default: without scheduling")
            .clicked()
        {
            send(out, TransferMsg::Mode(false));
        }
        if ui.radio(t.scheduling, "Advanced").clicked() {
            send(out, TransferMsg::Mode(true));
        }
        if t.scheduling {
            ui.add_space(4.0);
            ui.weak(
                "Advanced includes scheduling: the review history and the schedule (JSON only).",
            );
        }
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("Export").min_size(vec2(100.0, 32.0)))
                .clicked()
            {
                send(out, TransferMsg::Export);
            }
            ui.add_space(6.0);
            if ui
                .add(egui::Button::new("Back").min_size(vec2(80.0, 32.0)))
                .clicked()
            {
                send(out, TransferMsg::BackToExport);
            }
        });
    });
}

fn export_tab(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    if t.choosing_mode {
        mode_question(state, ui, out);
        return;
    }
    ui.add_space(6.0);
    kit::card(ui, |ui| {
        ui.label(
            RichText::new("Export Library or Stack")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.add_space(8.0);
        scope_menu(state, ui, out, "Export");
        ui.add_space(8.0);
        ui.label(RichText::new("Choose export format:").strong());
        ui.horizontal_wrapped(|ui| {
            for (f, label) in [
                (Format::Json, "JSON with pictures (zip)"),
                (Format::Toml, "TOML"),
                (Format::Ron, "RON"),
                (Format::Csv, "CSV (cards and questions)"),
                (Format::Markdown, "Markdown files (zip)"),
            ] {
                if ui.selectable_label(t.format == f, label).clicked() {
                    send(out, TransferMsg::Format(f));
                }
            }
        });
        ui.add_space(4.0);
        let note = match t.format {
            Format::Markdown => "Markdown files (import as an Obsidian vault or open directly).",
            _ => "Can be imported back into StudyFrog.",
        };
        ui.label(RichText::new(note).small().color(palette_of(ui).text_weak));
        ui.add_space(8.0);
        if let Some(text) = text_field(
            ui,
            "Write to folder",
            &t.destination,
            "/path/to/a/folder",
            1,
        ) {
            send(out, TransferMsg::Destination(text));
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("Choose folder…").min_size(vec2(120.0, 32.0)))
                .clicked()
            {
                send(out, TransferMsg::Browse(false, true));
            }
            ui.add_space(6.0);
            if ui
                .add_enabled(
                    !t.busy,
                    egui::Button::new("Export…").min_size(vec2(100.0, 32.0)),
                )
                .clicked()
            {
                send(out, TransferMsg::AskMode);
            }
            ui.add_space(6.0);
            if ui
                .add(egui::Button::new("Export to PDF…").min_size(vec2(120.0, 32.0)))
                .clicked()
            {
                let target = studyfrog_service::pdf_export::ExportTarget::Stack(
                    t.scope.clone().unwrap_or_default(),
                );
                out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Export(
                    crate::pdf_export_types::PdfExportMsg::Open(target),
                )));
            }
        });
    });
}

/// The dialog's content.
pub fn transfer(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;

    ui.horizontal(|ui| {
        ui.strong(
            RichText::new("Import and export")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading)),
        );
        for (tab, label) in [(Tab::Export, "Export"), (Tab::Import, "Import")] {
            if ui.selectable_label(t.tab == tab, label).clicked() {
                send(out, TransferMsg::Tab(tab));
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Cancel").clicked() {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });

    ui.add_space(4.0);
    ui.separator();

    match t.tab {
        Tab::Export => export_tab(state, ui, out),
        Tab::Import => super::modal_transfer_import::import_tab(state, ui, out),
    }
    if t.busy {
        ui.add_space(8.0);
        ui.spinner();
    }
    if let Some(s) = &t.status {
        ui.add_space(8.0);
        ui.label(RichText::new(s).color(palette_of(ui).text_weak));
    }
}
