//! The items of the selected stack (or the inbox), with a title filter and the new-card form.

use eframe::egui::{self, RichText};
use studyfrog_core::civil::format_date;

use crate::deadline_ui::deadline_of;
use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::state::{Selection, State};
use crate::theme::palette_of;

/// The page's heading: the stack's name, "Inbox" or "All items".
fn heading(state: &State) -> String {
    match &state.browser.selection {
        Selection::Stack(folder) => state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.iter().find(|s| s.folder == *folder))
            .map_or_else(|| folder.clone(), |s| s.title.clone()),
        Selection::Inbox => "Inbox".to_owned(),
        Selection::Nothing => "All items".to_owned(),
    }
}

/// Whether an extension offers a way to import, and a stack is open to import into.
fn can_import(state: &State) -> bool {
    let ext = &state.extensions;
    matches!(state.browser.selection, Selection::Stack(_))
        && !(ext.card_types.is_empty() && ext.importers.is_empty())
}

/// The search field, the count, the column menu and the table of items.
pub(super) fn list(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::library_search::draw(state, ui, out);
    ui.horizontal(|ui| {
        ui.label(crate::view::library_table::count_line(state));
        crate::view::library_table::column_menu(state, ui, out);
        crate::view::library_bulk::filters_menu(state, ui, out);
        if can_import(state) && ui.button("Import…").clicked() {
            out.push(Msg::Modal(crate::modal::ModalMsg::OpenImport));
        }
        if ui.button("Import or export…").clicked() {
            let scope = match &state.browser.selection {
                Selection::Stack(folder) => Some(folder.clone()),
                _ => None,
            };
            out.push(Msg::Library(crate::library_ui::LibraryMsg::Transfer(
                crate::transfer_ui::TransferMsg::Open(crate::transfer_ui::Tab::Import, scope),
            )));
        }
        crate::view::library_keys::button(state, ui, out);
    });
    crate::view::library_keys::confirm_bar(state, ui, out);
    crate::view::library_bulk::bar(state, ui, out);
    ui.allocate_ui(
        egui::vec2(
            ui.available_width(),
            (ui.available_height() - 16.0).max(260.0),
        ),
        |ui| {
            crate::view::library_table::draw(state, ui, out);
        },
    );
}

/// Heading and the list of items; for a stack also its due date, status roll-up, PDFs and the new-card form.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if matches!(state.browser.selection, Selection::Stack(_)) {
        crate::view::stack_view::draw(state, ui, out);
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(
                RichText::new(heading(state))
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong(),
            );
            let stack = matches!(state.browser.selection, Selection::Stack(_));
            if stack {
                due_row(state, ui, out);
                crate::view::status_ui::summary(ui, &state.browser.status_counts);
                child_stacks(state, ui, out);
            }
            list(state, ui, out);
            if let Selection::Stack(folder) = &state.browser.selection {
                crate::view::library_pdfs::stack_section(state, folder, ui, out);
            }
        });
}

/// The stack's due date and the button to set or change it.
pub(super) fn due_row(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (Selection::Stack(open), Some(vault)) = (&state.browser.selection, &state.vault) else {
        return;
    };
    let Some(stack) = vault.overview.stacks.iter().find(|s| s.folder == *open) else {
        return;
    };
    let deadline = deadline_of(&state.planner.sessions, stack.uid);
    let weak = palette_of(ui).text_weak;
    ui.horizontal(|ui| {
        let (text, button) = match deadline {
            Some(s) => (
                format!("Due {}", format_date(s.start.date)),
                "Change due date",
            ),
            None => ("No due date".to_owned(), "Set due date"),
        };
        ui.label(RichText::new(text).color(weak));
        if ui.small_button(button).clicked() {
            out.push(Msg::Plan(PlanMsg::StackDeadline(stack.uid)));
        }
        if ui.small_button("Edit stack").clicked() {
            out.push(Msg::OpenItem(stack.uid));
        }
    });
}

/// Buttons for the stacks inside the open stack.
pub(super) fn child_stacks(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (Selection::Stack(open), Some(vault)) = (&state.browser.selection, &state.vault) else {
        return;
    };
    let children: Vec<_> = vault
        .overview
        .stacks
        .iter()
        .filter(|s| s.parent.as_deref() == Some(open.as_str()))
        .collect();
    if children.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.weak("Stacks inside:");
        for c in children {
            if ui.button(&c.title).clicked() {
                out.push(Msg::Select(Selection::Stack(c.folder.clone())));
            }
        }
    });
}
