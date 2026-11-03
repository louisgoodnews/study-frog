//! The import and export dialog: pick the format and the scope, see the preview, choose what to do with duplicates, see the result.

use eframe::egui::{self, ComboBox};
use studyfrog_service::xfer_plan::{Policy, Verdict};

use crate::library_ui::LibraryMsg;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::transfer_ui::{Format, Tab, TransferMsg};
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: TransferMsg) {
    out.push(Msg::Library(LibraryMsg::Transfer(m)));
}

fn scope_menu(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>, label: &str) {
    let stacks: Vec<_> = state
        .vault
        .iter()
        .flat_map(|v| v.overview.stacks.iter())
        .collect();
    let current = state
        .transfer
        .scope
        .as_ref()
        .and_then(|f| stacks.iter().find(|s| s.folder == *f))
        .map_or_else(
            || {
                if label == "Export" {
                    "Whole library".to_owned()
                } else {
                    "As the file says".to_owned()
                }
            },
            |s| s.title.clone(),
        );
    let none = if label == "Export" {
        "Whole library"
    } else {
        "As the file says"
    };
    let name = if label == "Export" {
        "Export what"
    } else {
        "Import into"
    };
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
    ui.strong("Export mode");
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
        ui.weak("Advanced includes scheduling: the review history and the schedule (JSON only).");
    }
    ui.horizontal(|ui| {
        if ui.button("Export").clicked() {
            send(out, TransferMsg::Export);
        }
        if ui.button("Back").clicked() {
            send(out, TransferMsg::BackToExport);
        }
    });
}

fn export_tab(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    if t.choosing_mode {
        mode_question(state, ui, out);
        return;
    }
    scope_menu(state, ui, out, "Export");
    ui.horizontal(|ui| {
        for (f, label) in [
            (Format::Json, "JSON with pictures (zip)"),
            (Format::Csv, "CSV (cards and questions)"),
            (Format::Markdown, "Markdown files (zip)"),
        ] {
            if ui.selectable_label(t.format == f, label).clicked() {
                send(out, TransferMsg::Format(f));
            }
        }
    });
    if let Some(text) = text_field(
        ui,
        "Write to folder",
        &t.destination,
        "/path/to/a/folder",
        1,
    ) {
        send(out, TransferMsg::Destination(text));
    }
    ui.horizontal(|ui| {
        if ui.button("Choose folder…").clicked() {
            send(out, TransferMsg::Browse(false, true));
        }
        if ui
            .add_enabled(!t.busy, egui::Button::new("Export…"))
            .clicked()
        {
            send(out, TransferMsg::AskMode);
        }
    });
}

fn verdict(v: &Verdict) -> String {
    match v {
        Verdict::Create => "new".to_owned(),
        Verdict::Duplicate(p) => format!("exists at {p}"),
        Verdict::Error(e) => format!("error: {e}"),
    }
}

fn preview(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    let Some(p) = &t.preview else { return };
    let (create, dup, err) = p.counts;
    ui.strong(format!("{} found {} item(s)", p.adapter, p.total));
    ui.label(format!("{create} to create, {dup} already exist, {err} with errors, {} new stack(s), {} picture or PDF file(s)", p.new_stacks, p.files));
    for w in &p.warnings {
        ui.weak(w);
    }
    egui::ScrollArea::vertical()
        .max_height(160.0)
        .show(ui, |ui| {
            for l in &p.lines {
                ui.label(format!(
                    "{} ({}): {}",
                    l.title,
                    l.folder,
                    verdict(&l.verdict)
                ));
            }
        });
    if dup > 0 {
        ui.horizontal(|ui| {
            ui.label("Items that exist:");
            for (pol, label) in [
                (Policy::Skip, "Skip"),
                (Policy::Replace, "Replace"),
                (Policy::KeepBoth, "Keep both"),
            ] {
                if ui.selectable_label(t.policy == pol, label).clicked() {
                    send(out, TransferMsg::Policy(pol));
                }
            }
        });
    }
    if let Some(info) = &p.csv {
        super::modal_csv::settings(info, state.prompt_run.reply.is_some(), ui, out);
    }
    if p.scheduling {
        let mut on = t.with_schedule;
        if ui.checkbox(&mut on, "Import the review history").changed() {
            send(out, TransferMsg::WithSchedule(on));
        }
    }
    if ui
        .add_enabled(!t.busy && create + dup > 0, egui::Button::new("Import"))
        .clicked()
    {
        send(out, TransferMsg::Import);
    }
}

fn import_tab(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    super::modal_found::found(state, ui, out);
    super::modal_formats::formats(ui, out);
    if let Some(text) = text_field(ui, "File or folder to import", &t.path, "/path/to/file", 1) {
        send(out, TransferMsg::Path(text));
    }
    ui.horizontal(|ui| {
        if ui.button("Choose file…").clicked() {
            send(out, TransferMsg::Browse(true, false));
        }
        if ui.button("Choose folder…").clicked() {
            send(out, TransferMsg::Browse(true, true));
        }
    });
    scope_menu(state, ui, out, "Import");
    if ui
        .add_enabled(
            !t.busy && !t.path.trim().is_empty(),
            egui::Button::new("Preview"),
        )
        .clicked()
    {
        send(out, TransferMsg::Preview);
    }
    preview(state, ui, out);
}

/// The dialog's content.
pub fn transfer(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &state.transfer;
    ui.horizontal(|ui| {
        ui.strong("Import and export");
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
    ui.separator();
    match t.tab {
        Tab::Export => export_tab(state, ui, out),
        Tab::Import => import_tab(state, ui, out),
    }
    if t.busy {
        ui.spinner();
    }
    if let Some(s) = &t.status {
        ui.label(s);
    }
}
