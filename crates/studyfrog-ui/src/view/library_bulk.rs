//! The bar above the table: the Filters menu, saving a filter, and the actions on the selected rows.

mod menus;
use self::menus::{delete, menus, tags};
use eframe::egui::{self, Role, WidgetInfo};
use studyfrog_core::event::{Difficulty, difficulty_name};
use studyfrog_core::priority::{PRIORITIES, priority_name};
use studyfrog_core::status::{STATUSES, status_label};
use studyfrog_service::bulk::BulkEdit;

use crate::library_bulk::BulkMsg;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: BulkMsg) {
    out.push(Msg::Library(LibraryMsg::Bulk(m)));
}

/// The Filters menu: the saved filters (open or remove) and a field to save the current one.
pub fn filters_menu(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
    ui.menu_button("Filters", |ui| {
        if let Some(why) = &lib.filters_problem {
            ui.label(why);
        }
        if lib.filters.is_empty() {
            ui.weak("No saved filters yet.");
        }
        for f in &lib.filters {
            ui.horizontal(|ui| {
                if ui
                    .button(&f.name)
                    .on_hover_text("Opens this saved filter.")
                    .clicked()
                {
                    send(out, BulkMsg::OpenFilter(f.name.clone()));
                    ui.close();
                }
                if ui
                    .small_button(format!("Remove {}", f.name))
                    .on_hover_text("Removes this saved filter; the items are kept.")
                    .clicked()
                {
                    send(out, BulkMsg::DeleteFilter(f.name.clone()));
                }
            });
        }
        ui.separator();
        if let Some(text) = text_field(ui, "Name for this filter", &lib.filter_name, "", 1) {
            send(out, BulkMsg::FilterName(text));
        }
        if ui
            .button("Save current filter")
            .on_hover_text("Saves the current filters under a name.")
            .clicked()
        {
            send(out, BulkMsg::SaveFilter);
        }
    })
    .response
    .on_hover_text("Shows filters for the list.");
}

fn apply(out: &mut Vec<Msg>, edit: BulkEdit) {
    send(out, BulkMsg::Apply(edit));
}

/// The actions on the selected rows, and Undo for the last change.
pub fn bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
    if lib.undo.is_some()
        && ui
            .button("Undo last change")
            .on_hover_text("Undoes the last change to the reviews.")
            .clicked()
    {
        send(out, BulkMsg::Undo);
    }
    if lib.selected.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        let n = lib.selected.len();
        ui.label(format!("{n} selected"));
        menus(state, ui, out);
        tags(state, ui, out);
        if ui
            .button("Export to PDF…")
            .on_hover_text("Writes this as a PDF file you choose.")
            .clicked()
        {
            out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Export(
                crate::pdf_export_types::PdfExportMsg::Open(
                    studyfrog_service::pdf_export::ExportTarget::Selection(
                        lib.selected.iter().copied().collect(),
                    ),
                ),
            )));
        }
        if (2..=4).contains(&n)
            && ui
                .button("Contrastive drill")
                .on_hover_text("Starts a drill that compares two items.")
                .clicked()
        {
            let picked = lib.selected.iter().copied().collect();
            out.push(crate::contrast::msg(crate::contrast::ContrastMsg::Open(
                picked,
            )));
        }
        delete(state, ui, out);
        if ui
            .button("Clear selection")
            .on_hover_text("Clears the selection.")
            .clicked()
        {
            send(out, BulkMsg::SelectAll(false));
        }
    });
}
