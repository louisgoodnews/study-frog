//! The page of a stack: breadcrumb, title, status and actions on top, then its tabs.

use eframe::egui::{self, Key, RichText};
use studyfrog_store::overview::StackRow;

use crate::home_ui::HomeMsg;
use crate::library_ui::LibraryMsg;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::stack_view_ui::{StackTab, StackViewMsg, TABS, tab_of};
use crate::state::{Selection, State};
use crate::view::kit;
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
        if ui.button("New item").clicked() {
            out.push(Msg::Modal(ModalMsg::OpenCreate));
        }
        if kit::primary_button(ui, "Start rehearsal").clicked() {
            out.push(Msg::Home(HomeMsg::RehearseStack(stack.folder.clone())));
        }
        more_actions(state, stack, ui, out);
    });
}

/// The rarely used actions of a stack, out of the way in a menu.
fn more_actions(state: &State, stack: &StackRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.menu_button("More actions", |ui| {
        if ui.button("Details").clicked() {
            out.push(Msg::OpenItem(stack.uid));
            ui.close();
        }
        let ext = &state.extensions;
        if !(ext.card_types.is_empty() && ext.importers.is_empty())
            && ui.button("Import…").clicked()
        {
            out.push(Msg::Modal(ModalMsg::OpenImport));
            ui.close();
        }
        if ui.button("Import or export…").clicked() {
            let open = crate::transfer_ui::TransferMsg::Open(
                crate::transfer_ui::Tab::Export,
                Some(stack.folder.clone()),
            );
            out.push(Msg::Library(LibraryMsg::Transfer(open)));
            ui.close();
        }
        if ui.button("Export to PDF…").clicked() {
            out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Export(
                crate::pdf_export_types::PdfExportMsg::Open(
                    studyfrog_service::pdf_export::ExportTarget::Stack(stack.folder.clone()),
                ),
            )));
            ui.close();
        }
        if ui.button("Make normal folder…").clicked() {
            out.push(Msg::Modal(ModalMsg::AskUnmark(stack.folder.clone())));
            ui.close();
        }
        move_stack_menu(state, stack, ui, out);
    });
}

fn move_stack_menu(state: &State, stack: &StackRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(vault) = &state.vault else { return };
    let targets: Vec<_> = vault
        .overview
        .stacks
        .iter()
        .filter(|s| {
            s.folder != stack.folder
                && !studyfrog_service::folder_ops::is_subpath_or_equal(&stack.folder, &s.folder)
                && Some(s.folder.as_str()) != stack.parent.as_deref()
        })
        .collect();
    if stack.parent.is_none() && targets.is_empty() {
        return;
    }
    ui.menu_button("Move stack…", |ui| {
        if stack.parent.is_some() && ui.button("Make standalone (top-level)").clicked() {
            send(
                out,
                StackViewMsg::MoveStack(stack.folder.clone(), String::new()),
            );
            ui.close();
        }
        for s in targets {
            if ui.button(format!("Move under {}", s.title)).clicked() {
                send(
                    out,
                    StackViewMsg::MoveStack(stack.folder.clone(), s.folder.clone()),
                );
                ui.close();
            }
        }
    });
}

const TAB_WIDTH: f32 = 84.0;
const MORE_WIDTH: f32 = 70.0;

/// How many tabs show in the bar for `width`, always the current one among them; the others go into More.
fn shown_tabs(width: f32, current: StackTab) -> Vec<StackTab> {
    let fit = (((width - MORE_WIDTH) / TAB_WIDTH).floor() as usize).max(1);
    if fit >= TABS.len() {
        return TABS.iter().map(|(t, _, _)| *t).collect();
    }
    let mut shown: Vec<StackTab> = TABS.iter().take(fit).map(|(t, _, _)| *t).collect();
    if !shown.contains(&current) {
        shown.pop();
        shown.push(current);
    }
    shown
}

/// The tab bar with a More menu for the tabs that do not fit, and the keys Left and Right.
fn tab_bar(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let current = tab_of(&state.app, folder);
    let shown = shown_tabs(ui.available_width(), current);
    ui.horizontal(|ui| {
        for (tab, _, label) in TABS.iter().filter(|(t, _, _)| shown.contains(t)) {
            if ui.selectable_label(*tab == current, *label).clicked() {
                send(out, StackViewMsg::Select(*tab));
            }
        }
        if shown.len() < TABS.len() {
            ui.menu_button("More", |ui| {
                for (tab, _, label) in TABS.iter().filter(|(t, _, _)| !shown.contains(t)) {
                    if ui.button(*label).clicked() {
                        send(out, StackViewMsg::Select(*tab));
                        ui.close();
                    }
                }
            });
        }
    });
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    for (key, by) in [(Key::ArrowLeft, -1), (Key::ArrowRight, 1)] {
        // Alt+Left and Alt+Right are Back and Forward in the history, not tab keys.
        if ui.input(|i| i.key_pressed(key) && !i.modifiers.alt) {
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
                StackTab::Roadmap => crate::view::stack_roadmap_view::draw(state, folder, ui, out),
                StackTab::Board => crate::view::stack_board_view::draw(state, folder, ui, out),
                StackTab::Calendar => {
                    let scope = stack.uid;
                    crate::view::calendar::draw_scoped(state, Some(scope), ui, out);
                }
                StackTab::Reports => crate::view::stack_reports_view::draw(state, folder, ui, out),
                StackTab::Goals => crate::view::stack_goals_view::draw(state, folder, ui, out),
                StackTab::Substacks => crate::view::stack_tree_view::draw(state, folder, ui, out),
                StackTab::Backlog => crate::view::stack_backlog_view::draw(state, folder, ui, out),
            }
        });
}

fn list_tab(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    super::items::due_row(state, ui, out);
    super::items::child_stacks(state, ui, out);
    super::items::list(state, ui, out);
    crate::view::library_pdfs::stack_section(state, folder, ui, out);
}
