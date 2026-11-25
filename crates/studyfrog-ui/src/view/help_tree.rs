//! The page tree of the help screen (EPIC-073, STORY-0227): the FrogScript reference, the groups of supplied pages and
//! the user's own pages, each group on a redesign card. The same entries and actions as before.
use eframe::egui;
use studyfrog_service::help::{PAGES, PageRef, SECTIONS, title_of, user_title};

use crate::console_ui::ConsoleMsg;
use crate::help_types::HelpMsg;
use crate::help_ui::current;
use crate::msg::{Msg, MsgVault};
use crate::script_studio::wiki_panel::WikiMsg;
use crate::state::{Screen, State};
use crate::view::{home_hero, kit};

fn send(out: &mut Vec<Msg>, m: HelpMsg) {
    out.push(Msg::Help(m));
}

fn entry(ui: &mut egui::Ui, open: bool, title: &str, page: PageRef, out: &mut Vec<Msg>) {
    let width = ui.available_width();
    let button = egui::Button::selectable(open, title).min_size(egui::vec2(width, 24.0));
    if ui
        .add(button)
        .on_hover_text("Opens this help page.")
        .clicked()
    {
        send(out, HelpMsg::Open(page));
    }
}

/// The whole tree: one card for the FrogScript reference, one per group of pages, one for the user's pages.
pub(super) fn tree(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let shown = current(&state.help);
    home_hero::card(ui, |ui| frogscript(state, ui, out));
    ui.add_space(6.0);
    for (section, name) in SECTIONS {
        home_hero::card(ui, |ui| {
            kit::caption(ui, name);
            for p in PAGES.iter().filter(|p| p.section == section) {
                let page = PageRef::Supplied(p.id.to_owned());
                entry(ui, shown == page, title_of(p.text, p.id), page, out);
            }
        });
        ui.add_space(6.0);
    }
    home_hero::card(ui, |ui| user_pages(state, &shown, ui, out));
    ui.add_space(6.0);
    crate::view::tutorial::tutorials(state, ui, out);
}

fn frogscript(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::caption(ui, "FrogScript");
    if state.vault.is_none() {
        ui.weak("Open a vault to use the FrogScript reference.");
    } else if ui
        .button("FrogScript reference")
        .on_hover_text("Opens the reference of FrogScript in ScriptStudio.")
        .clicked()
    {
        out.push(Msg::Console(ConsoleMsg::Wiki(WikiMsg::Show)));
        out.push(Msg::Vault(MsgVault::Navigate(Screen::Console)));
    }
}

fn user_pages(state: &State, shown: &PageRef, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::caption(ui, "Your pages");
    for u in &state.help.user {
        let page = PageRef::User(u.file.clone());
        entry(ui, *shown == page, user_title(u), page, out);
    }
    if state.vault.is_none() {
        ui.weak("Open a vault to keep your own pages.");
        return;
    }
    if ui
        .button("+ New page")
        .on_hover_text("Adds a new page.")
        .clicked()
    {
        send(out, HelpMsg::New);
    }
    if ui
        .button("Ask the assistant")
        .on_hover_text("The assistant reads your pages (after you press Send) and proposes new pages or edits.")
        .clicked()
    {
        let on = crate::agent_types::AgentMsg::UseHelpPages(true);
        out.push(Msg::Agent(on));
        out.push(Msg::Vault(MsgVault::Navigate(Screen::Agent)));
    }
}
