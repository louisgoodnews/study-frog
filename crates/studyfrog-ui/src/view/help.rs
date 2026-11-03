//! The help screen: search and the page tree on the left, the page on the right.

use eframe::egui::{self, RichText, ScrollArea, TextEdit};
use studyfrog_service::help::{Hit, PAGES, PageRef, SECTIONS, title_of, user_title};

use crate::help_types::{HelpMsg, HelpState};
use crate::help_ui::current;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;
use crate::view::kit;

const TREE_WIDTH: f32 = 230.0;

fn send(out: &mut Vec<Msg>, m: HelpMsg) {
    out.push(Msg::Help(m));
}

/// The name of a screen as the sidebar shows it.
pub fn screen_name(screen: Screen) -> &'static str {
    match screen {
        Screen::Welcome => "the vault chooser",
        Screen::Home => "Home",
        Screen::Library => "Library",
        Screen::Rehearse => "Rehearse",
        Screen::Settings => "Settings",
        Screen::Editor => "the item",
        Screen::Graph => "Graph",
        Screen::Calendar => "Calendar",
        Screen::Board => "Board",
        Screen::Achievements => "Achievements",
        Screen::Reports => "Reports",
        Screen::Agent => "Assistant",
        Screen::Pdf => "the PDF",
        Screen::Help => "Help",
    }
}

/// The screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let help = &state.help;
    egui::Panel::left("help-tree")
        .resizable(false)
        .exact_size(TREE_WIDTH)
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("help-tree")
                .show(ui, |ui| side(state, ui, out));
        });
    egui::CentralPanel::default().show(ui, |ui| {
        ScrollArea::vertical()
            .id_salt("help-page")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                kit::column(ui, kit::READING_WIDTH, |ui| page(state, help, ui, out));
            });
    });
}

fn side(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let help = &state.help;
    let mut text = help.search.clone();
    let field = ui.add(
        TextEdit::singleline(&mut text)
            .hint_text("Search help")
            .desired_width(f32::INFINITY),
    );
    field.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Search help"));
    if text != help.search {
        send(out, HelpMsg::Search(text));
    }
    ui.add_space(6.0);
    if help.search.trim().is_empty() {
        tree(state, ui, out);
    } else {
        hits(&help.hits, &help.search, ui, out);
    }
}

fn entry(ui: &mut egui::Ui, open: bool, title: &str, page: PageRef, out: &mut Vec<Msg>) {
    let width = ui.available_width();
    let button = egui::Button::selectable(open, title).min_size(egui::vec2(width, 24.0));
    if ui.add(button).clicked() {
        send(out, HelpMsg::Open(page));
    }
}

fn tree(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let shown = current(&state.help);
    for (section, name) in SECTIONS {
        kit::caption(ui, name);
        for p in PAGES.iter().filter(|p| p.section == section) {
            let page = PageRef::Supplied(p.id.to_owned());
            entry(ui, shown == page, title_of(p.text, p.id), page, out);
        }
        ui.add_space(6.0);
    }
    kit::caption(ui, "Your pages");
    for u in &state.help.user {
        let page = PageRef::User(u.file.clone());
        entry(ui, shown == page, user_title(u), page, out);
    }
    if state.vault.is_none() {
        ui.weak("Open a vault to keep your own pages.");
    } else {
        if ui.button("+ New page").clicked() {
            send(out, HelpMsg::New);
        }
        if ui
            .button("Ask the assistant")
            .on_hover_text("The assistant reads your pages (after you press Send) and proposes new pages or edits.")
            .clicked()
        {
            let on = crate::agent_types::AgentMsg::UseHelpPages(true);
            out.push(Msg::Agent(on));
            out.push(Msg::Navigate(Screen::Agent));
        }
    }
    ui.add_space(6.0);
    crate::view::tutorial::tutorials(state, ui, out);
}

fn hits(hits: &[Hit], query: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if hits.is_empty() {
        ui.weak(format!("No page mentions \"{}\".", query.trim()));
    }
    for h in hits {
        let text = format!("{}\n{}", h.title, h.snippet);
        let width = ui.available_width();
        let button = egui::Button::new(text)
            .min_size(egui::vec2(width, 0.0))
            .wrap();
        if ui.add(button).clicked() {
            send(out, HelpMsg::Open(h.page.clone()));
        }
    }
}

fn top_row(state: &State, help: &HelpState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if !help.back.is_empty() && ui.button("Previous page").clicked() {
            send(out, HelpMsg::Back);
        }
        let from = help.came_from.filter(|s| *s != Screen::Help);
        if let Some(from) = from
            && ui
                .button(format!("Back to {}", screen_name(from)))
                .clicked()
        {
            send(out, HelpMsg::Leave);
        }
        if help.edit.is_some() || state.vault.is_none() {
            return;
        }
        match current(help) {
            PageRef::Supplied(_) => {
                if ui
                    .button("Copy to my pages")
                    .on_hover_text("Supplied pages are read-only; a copy is yours to change.")
                    .clicked()
                {
                    send(out, HelpMsg::Copy);
                }
            }
            PageRef::User(_) => user_actions(help, ui, out),
        }
    });
}

fn user_actions(help: &HelpState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if ui.button("Edit").clicked() {
        send(out, HelpMsg::Edit);
    }
    if !help.ask_delete {
        if ui.button("Delete").clicked() {
            send(out, HelpMsg::AskDelete(true));
        }
        return;
    }
    ui.label("Delete this page for good?");
    if ui.button("Delete page").clicked() {
        send(out, HelpMsg::Delete);
    }
    if ui.button("Keep it").clicked() {
        send(out, HelpMsg::AskDelete(false));
    }
}

fn page(state: &State, help: &HelpState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    top_row(state, help, ui, out);
    ui.add_space(4.0);
    if let Some(edit) = &help.edit {
        crate::view::help_edit::editor(edit, ui, out);
        return;
    }
    if help.blocks.is_empty() {
        let weak = palette_of(ui).text_weak;
        ui.label(RichText::new("This page no longer exists.").color(weak));
        return;
    }
    let mut links = Vec::new();
    crate::view::preview::draw_blocks(ui, &help.blocks, &mut links);
    out.extend(links.into_iter().map(|m| match m {
        Msg::OpenLink(target) => Msg::Help(HelpMsg::Link(target)),
        other => other,
    }));
}
