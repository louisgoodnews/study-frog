//! The page of a stack: breadcrumb, title, status and actions on top, then its tabs.

use eframe::egui::{self, Key, RichText};
use studyfrog_store::overview::StackRow;

use crate::home_ui::HomeMsg;
use crate::library_ui::LibraryMsg;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::stack_view_ui::{StackTab, StackViewMsg, TABS, tab_of};
use crate::state::{Selection, State};
use crate::view::status_ui::status_menu;

fn send(out: &mut Vec<Msg>, m: StackViewMsg) {
    out.push(Msg::Library(LibraryMsg::Stack(m)));
}

/// The stacks above `stack`, outermost first.
fn ancestors<'a>(stacks: &'a [StackRow], stack: &StackRow) -> Vec<&'a StackRow> {
    let mut chain = Vec::new();
    let mut at = stack.parent.as_deref();
    while let Some(folder) = at {
        let Some(s) = stacks.iter().find(|s| s.folder == folder) else {
            break;
        };
        chain.push(s);
        at = s.parent.as_deref();
    }
    chain.reverse();
    chain
}

fn breadcrumb(stacks: &[StackRow], stack: &StackRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if ui.link("All items").clicked() {
            out.push(Msg::Select(Selection::Nothing));
        }
        for a in ancestors(stacks, stack) {
            ui.label("/");
            if ui.link(&a.title).clicked() {
                out.push(Msg::Select(Selection::Stack(a.folder.clone())));
            }
        }
    });
}

fn actions(state: &State, stack: &StackRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if let Some(pick) = status_menu(ui, stack.status, None) {
            send(out, StackViewMsg::SetStatus(stack.uid, pick));
        }
        if ui.button("Details").clicked() {
            out.push(Msg::OpenItem(stack.uid));
        }
        if ui.button("New item").clicked() {
            out.push(Msg::Modal(ModalMsg::OpenCreate));
        }
        let ext = &state.extensions;
        if !(ext.card_types.is_empty() && ext.importers.is_empty())
            && ui.button("Import…").clicked()
        {
            out.push(Msg::Modal(ModalMsg::OpenImport));
        }
        if ui.button("Start rehearsal").clicked() {
            out.push(Msg::Home(HomeMsg::RehearseStack(stack.folder.clone())));
        }
    });
}

/// The tab bar with the keys Left and Right; returns nothing, sends the choice.
fn tab_bar(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let current = tab_of(&state.app, folder);
    ui.horizontal(|ui| {
        for (tab, _, label) in TABS {
            if ui.selectable_label(tab == current, label).clicked() {
                send(out, StackViewMsg::Select(tab));
            }
        }
    });
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    for (key, by) in [(Key::ArrowLeft, -1), (Key::ArrowRight, 1)] {
        if ui.input(|i| i.key_pressed(key)) {
            send(out, StackViewMsg::Step(by));
        }
    }
}

/// The page.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Selection::Stack(folder) = &state.browser.selection else {
        return;
    };
    let Some(vault) = &state.vault else { return };
    let Some(stack) = vault.overview.stacks.iter().find(|s| s.folder == *folder) else {
        return;
    };
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            breadcrumb(&vault.overview.stacks, stack, ui, out);
            ui.label(RichText::new(&stack.title).size(26.0).strong());
            actions(state, stack, ui, out);
            crate::view::status_ui::summary(ui, &state.browser.status_counts);
            ui.add_space(6.0);
            tab_bar(state, folder, ui, out);
            ui.separator();
            match tab_of(&state.app, folder) {
                StackTab::Summary => crate::view::stack_summary_view::draw(state, folder, ui, out),
                StackTab::List => list_tab(state, folder, ui, out),
            }
        });
}

fn list_tab(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    super::items::due_row(state, ui, out);
    super::items::child_stacks(state, ui, out);
    super::items::list(state, ui, out);
    crate::view::library_pdfs::stack_section(state, folder, ui, out);
}
