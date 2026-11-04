//! The table of items under the search bar: sortable columns, a count and rows that are only built when they scroll into view.

use eframe::egui::{self, Align, Id, Layout, RichText, Role, WidgetInfo};
use egui_extras::{Column as Col, TableBuilder};
use studyfrog_core::query::Order;

use crate::library_bulk::BulkMsg;
use crate::library_ui::{COLUMNS, Column, LibraryMsg, cell, column_field, column_label};
use crate::msg::Msg;
use crate::state::{Selection, State};
use crate::theme::palette_of;
use studyfrog_service::library_rows::LibraryItem;

const ROW_HEIGHT: f32 = 24.0;

fn send(out: &mut Vec<Msg>, m: LibraryMsg) {
    out.push(Msg::Library(m));
}

fn width(c: Column) -> f32 {
    match c {
        Column::Title => 300.0,
        Column::Path | Column::Stack | Column::AlsoIn | Column::Subjects | Column::Tags => 180.0,
        Column::Type | Column::Reviews => 80.0,
        _ => 110.0,
    }
}

/// The font has no arrow glyphs, so the direction is a plain letter: `^` ascending, `v` descending.
fn arrow(sort: &[Order], c: Column) -> &'static str {
    match column_field(c).and_then(|f| sort.iter().find(|o| o.field == f)) {
        Some(o) if o.descending => " v",
        Some(_) => " ^",
        None => "",
    }
}

fn heading(state: &State, ui: &mut egui::Ui, c: Column, out: &mut Vec<Msg>) {
    let label = format!("{}{}", column_label(c), arrow(&state.library.sort, c));
    if column_field(c).is_none() {
        ui.label(RichText::new(label).strong());
        return;
    }
    let r = ui.add(egui::Button::new(RichText::new(label).strong()).frame(false));
    let state_word = match arrow(&state.library.sort, c) {
        " ^" => ", sorted ascending",
        " v" => ", sorted descending",
        _ => "",
    };
    let name = format!("Sort by {}{state_word}", column_label(c));
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, name.clone()));
    if r.clicked() {
        send(out, LibraryMsg::Sort(c, ui.input(|i| i.modifiers.shift)));
    }
}

/// The "N of M items" line.
pub fn count_line(state: &State) -> String {
    let lib = &state.library;
    format!("{} of {} items", lib.shown.len(), lib.scope_total)
}

/// The column menu: which columns are shown.
pub fn column_menu(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.menu_button("Columns", |ui| {
        for (c, _, label) in COLUMNS {
            let mut on = state.library.columns.contains(&c);
            let r = ui.add_enabled(c != Column::Title, egui::Checkbox::new(&mut on, label));
            if r.changed() {
                send(out, LibraryMsg::ToggleColumn(c));
            }
        }
    });
}

fn tick(ui: &mut egui::Ui, on: bool, name: String) -> Option<bool> {
    let mut value = on;
    let r = ui.add(egui::Checkbox::without_text(&mut value));
    r.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, on, name.clone()));
    r.changed().then_some(value)
}

fn select_all_box(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
    let all = !lib.shown.is_empty() && lib.selected.len() >= lib.shown.len();
    if let Some(on) = tick(ui, all, "Select all rows".to_owned()) {
        send(out, LibraryMsg::Bulk(BulkMsg::SelectAll(on)));
    }
}

fn row_box(state: &State, item: &LibraryItem, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let on = state.library.selected.contains(&item.uid);
    if let Some(v) = tick(ui, on, format!("Select {}", item.row.title)) {
        send(out, LibraryMsg::Bulk(BulkMsg::Select(item.uid, v)));
    }
}

/// The table.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let lib = &state.library;
    if lib.shown.is_empty() {
        let text = if lib.scope_total == 0 {
            "No items here yet."
        } else {
            "No items match. Clear the search?"
        };
        ui.label(text);
        return;
    }
    let weak = palette_of(ui).text_weak;
    let mut table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(Layout::left_to_right(Align::Center))
        .max_scroll_height(f32::INFINITY);
    table = table.column(Col::exact(28.0));
    for c in &lib.columns {
        table = table.column(Col::initial(width(*c)).at_least(60.0).clip(true));
    }
    table
        .header(ROW_HEIGHT, |mut header| {
            header.col(|ui| select_all_box(state, ui, out));
            for c in &lib.columns {
                header.col(|ui| heading(state, ui, *c, out));
            }
        })
        .body(|body| {
            body.rows(ROW_HEIGHT, lib.shown.len(), |mut row| {
                let item = &lib.items[lib.shown[row.index()]];
                row.col(|ui| row_box(state, item, ui, out));
                for c in &lib.columns {
                    row.col(|ui| {
                        let text = cell(&item.row, *c);
                        if *c == Column::Title {
                            ui.dnd_drag_source(
                                Id::unique(("library-item", item.uid)),
                                item.uid,
                                |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.link(&text).clicked() {
                                            out.push(Msg::OpenItem(item.uid));
                                        }
                                        if let Selection::Stack(folder) = &state.browser.selection
                                            && !item
                                                .row
                                                .stacks
                                                .iter()
                                                .step_by(2)
                                                .any(|s| s == folder)
                                        {
                                            let from = item
                                                .references
                                                .iter()
                                                .find(|r| r.primary)
                                                .map_or("Inbox", |r| r.title.as_str());
                                            ui.weak(format!("(from {from})"));
                                        }
                                    });
                                },
                            );
                        } else {
                            let label = ui.label(RichText::new(text).color(weak));
                            if *c == Column::Familiarity && item.references.len() > 1 {
                                label.on_hover_text(studyfrog_service::fam_refs::per_stack(
                                    &item.references,
                                ));
                            }
                        }
                    });
                }
            });
        });
}
