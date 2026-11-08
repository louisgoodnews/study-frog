//! The bar above the table: the Filters menu, saving a filter, and the actions on the selected rows.

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
                if ui.button(&f.name).clicked() {
                    send(out, BulkMsg::OpenFilter(f.name.clone()));
                    ui.close();
                }
                if ui.small_button(format!("Remove {}", f.name)).clicked() {
                    send(out, BulkMsg::DeleteFilter(f.name.clone()));
                }
            });
        }
        ui.separator();
        if let Some(text) = text_field(ui, "Name for this filter", &lib.filter_name, "", 1) {
            send(out, BulkMsg::FilterName(text));
        }
        if ui.button("Save current filter").clicked() {
            send(out, BulkMsg::SaveFilter);
        }
    });
}

fn apply(out: &mut Vec<Msg>, edit: BulkEdit) {
    send(out, BulkMsg::Apply(edit));
}

fn menus(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.menu_button("Set status", |ui| {
        for s in STATUSES {
            if ui.button(status_label(s)).clicked() {
                apply(out, BulkEdit::Status(s));
                ui.close();
            }
        }
    });
    ui.menu_button("Set priority", |ui| {
        for p in PRIORITIES {
            if ui.button(priority_name(p)).clicked() {
                apply(out, BulkEdit::Priority(p));
                ui.close();
            }
        }
    });
    ui.menu_button("Set difficulty", |ui| {
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Difficult] {
            if ui.button(difficulty_name(d)).clicked() {
                apply(out, BulkEdit::Difficulty(d));
                ui.close();
            }
        }
    });
    let stacks_move = state.vault.iter().flat_map(|v| v.overview.stacks.iter());
    ui.menu_button("Move to stack", |ui| {
        for s in stacks_move {
            let r = ui.button(&s.title);
            r.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, format!("Move to stack {}", s.title))
            });
            if r.clicked() {
                apply(out, BulkEdit::MoveTo(s.folder.clone()));
                ui.close();
            }
        }
    });
    let stacks_add = state.vault.iter().flat_map(|v| v.overview.stacks.iter());
    ui.menu_button("Add to stack", |ui| {
        for s in stacks_add {
            let r = ui.button(&s.title);
            r.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, format!("Add to stack {}", s.title))
            });
            if r.clicked() {
                apply(out, BulkEdit::AddTo(s.folder.clone()));
                ui.close();
            }
        }
    });
}

fn tags(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let tag = state.library.tag_input.trim().to_owned();
    if let Some(text) = text_field(ui, "Tag", &state.library.tag_input, "", 1) {
        send(out, BulkMsg::TagInput(text));
    }
    let on = !tag.is_empty();
    if ui.add_enabled(on, egui::Button::new("Add tag")).clicked() {
        apply(out, BulkEdit::AddTag(tag.clone()));
    }
    if ui
        .add_enabled(on, egui::Button::new("Remove tag"))
        .clicked()
    {
        apply(out, BulkEdit::RemoveTag(tag));
    }
}

fn delete(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let n = state.library.selected.len();
    if !state.library.confirm_delete {
        if ui.button("Delete").clicked() {
            apply(out, BulkEdit::Delete);
        }
        return;
    }
    ui.label(format!(
        "Delete {n} selected item{}?",
        if n == 1 { "" } else { "s" }
    ));
    if ui.button("Yes, delete").clicked() {
        apply(out, BulkEdit::Delete);
    }
    if ui.button("Keep them").clicked() {
        send(out, BulkMsg::AskDelete(false));
    }
}

/// The actions on the selected rows, and Undo for the last change.
pub fn bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
    if lib.undo.is_some() && ui.button("Undo last change").clicked() {
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
        if ui.button("Export to PDF…").clicked() {
            out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Export(
                crate::pdf_export_types::PdfExportMsg::Open(
                    studyfrog_service::pdf_export::ExportTarget::Selection(
                        lib.selected.iter().copied().collect(),
                    ),
                ),
            )));
        }
        if (2..=4).contains(&n) && ui.button("Contrastive drill").clicked() {
            let picked = lib.selected.iter().copied().collect();
            out.push(crate::contrast::msg(crate::contrast::ContrastMsg::Open(
                picked,
            )));
        }
        delete(state, ui, out);
        if ui.button("Clear selection").clicked() {
            send(out, BulkMsg::SelectAll(false));
        }
    });
}
