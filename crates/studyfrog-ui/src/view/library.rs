//! The library screen: stack list on the left, overview or the selected stack on the right.

use eframe::egui::{self, Panel, RichText};
use studyfrog_core::scheduler::{Scheduler, scheduler_name};
use studyfrog_store::overview::{StackRow, VaultOverview};

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::{Selection, State};
use crate::theme::palette_of;
use crate::view::{items, kit};

fn limit(n: Option<u32>) -> String {
    n.map_or_else(|| "–".to_owned(), |v| v.to_string())
}

fn scheduler_text(row: &StackRow, default: Scheduler) -> String {
    match row.scheduler {
        Some(s) => scheduler_name(s).to_owned(),
        None => format!("{} (default)", scheduler_name(default)),
    }
}

/// Both halves of the screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.vault.is_none() {
        return;
    }
    Panel::left("stack-list")
        .default_size(220.0)
        .show(ui, |ui| stack_list(state, ui, out));
    match &state.browser.selection {
        Selection::Nothing => overview(state, ui, out),
        _ => items::draw(state, ui, out),
    }
}

/// Nesting depth of each stack: how many stacks it sits inside.
fn depth(stacks: &[StackRow], s: &StackRow) -> usize {
    let mut n = 0;
    let mut up = s.parent.as_deref();
    while let Some(p) = up {
        n += 1;
        up = stacks
            .iter()
            .find(|x| x.folder == p)
            .and_then(|x| x.parent.as_deref());
    }
    n
}

fn stack_list(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(vault) = &state.vault else { return };
    ui.heading("Stacks");
    let current = &state.browser.selection;
    for s in &vault.overview.stacks {
        let target = Selection::Stack(s.folder.clone());
        ui.horizontal(|ui| {
            ui.add_space(14.0 * depth(&vault.overview.stacks, s) as f32);
            if ui.selectable_label(*current == target, &s.title).clicked() {
                out.push(Msg::Select(target));
            }
        });
    }
    let inbox = vault.overview.inbox_items;
    if inbox > 0
        && ui
            .selectable_label(*current == Selection::Inbox, format!("Inbox ({inbox})"))
            .clicked()
    {
        out.push(Msg::Select(Selection::Inbox));
    }
    if ui
        .selectable_label(*current == Selection::Nothing, "Overview")
        .clicked()
    {
        out.push(Msg::Select(Selection::Nothing));
    }
    ui.separator();
    if ui.button("New stack…").clicked() {
        out.push(Msg::Modal(ModalMsg::OpenCreateStack));
    }
}

/// Title, counts and a grid of stack cards; any problems below it.
fn overview(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(vault) = &state.vault else { return };
    let o = &vault.overview;
    let items: usize = o
        .stacks
        .iter()
        .map(|s| s.notes + s.cards + s.questions)
        .sum::<usize>()
        + o.inbox_items;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(RichText::new("All stacks").size(26.0).strong());
        ui.label(
            RichText::new(format!("{} stacks, {items} items", o.stacks.len()))
                .color(palette_of(ui).text_weak),
        );
        ui.add_space(8.0);
        if o.stacks.is_empty() {
            ui.label(
                "No stacks yet. Use Create, or add a folder with a _stack.md file to the vault.",
            );
        }
        stack_cards(o, vault.settings.scheduler, ui, out);
        if o.inbox_items > 0 {
            ui.add_space(6.0);
            ui.label(format!("Inbox: {} items outside any stack", o.inbox_items));
        }
        problems(o, ui);
    });
}

const COLUMNS: usize = 3;

fn stack_cards(o: &VaultOverview, default: Scheduler, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for row in o.stacks.chunks(COLUMNS) {
        ui.columns(COLUMNS, |cols| {
            for (col, s) in cols.iter_mut().zip(row) {
                let id = format!("stack-card-{}", s.folder);
                if kit::card_button(col, &id, |ui| stack_card(o, s, default, ui)) {
                    out.push(Msg::Select(Selection::Stack(s.folder.clone())));
                }
            }
        });
        ui.add_space(4.0);
    }
}

fn stack_card(o: &VaultOverview, s: &StackRow, default: Scheduler, ui: &mut egui::Ui) {
    let p = palette_of(ui);
    ui.set_min_height(86.0);
    ui.label(RichText::new(&s.title).strong().size(17.0));
    let place = match &s.parent {
        Some(f) => o
            .stacks
            .iter()
            .find(|x| x.folder == *f)
            .map_or("inside another stack", |x| x.title.as_str()),
        None => "",
    };
    if !place.is_empty() {
        ui.label(
            RichText::new(format!("in {place}"))
                .small()
                .color(p.text_weak),
        );
    }
    ui.horizontal_wrapped(|ui| {
        for (n, word, color) in [
            (s.cards, "cards", p.accent),
            (s.notes, "notes", p.text_weak),
            (s.questions, "questions", p.text_weak),
        ] {
            if n > 0 {
                kit::pill(ui, &format!("{n} {word}"), color);
            }
        }
        if s.cards + s.notes + s.questions == 0 {
            kit::pill(ui, "empty", p.text_weak);
        }
    });
    ui.label(
        RichText::new(format!(
            "{} · {} new / {} reviews a day",
            scheduler_text(s, default),
            limit(s.daily_new_limit),
            limit(s.daily_review_limit)
        ))
        .small()
        .color(p.text_weak),
    );
}

fn problems(o: &VaultOverview, ui: &mut egui::Ui) {
    if o.problems.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.collapsing(format!("Problems ({})", o.problems.len()), |ui| {
        for p in &o.problems {
            ui.label(format!("{}: {}", p.path, p.message));
        }
    });
}
