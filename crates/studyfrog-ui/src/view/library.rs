//! The library screen: stack list on the left, overview or the selected stack on the right.

use eframe::egui::{self, Grid, Panel};
use studyfrog_core::scheduler::{Scheduler, scheduler_name};
use studyfrog_store::overview::{StackRow, VaultOverview};

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::{Selection, State};
use crate::view::items;

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
        Selection::Nothing => overview(state, ui),
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

/// Title, counts and a table of stacks; any problems below it.
fn overview(state: &State, ui: &mut egui::Ui) {
    let Some(vault) = &state.vault else { return };
    ui.heading(&vault.name);
    let o = &vault.overview;
    let items: usize = o
        .stacks
        .iter()
        .map(|s| s.notes + s.cards + s.questions)
        .sum::<usize>()
        + o.inbox_items;
    ui.label(format!("{} stacks, {items} items", o.stacks.len()));
    ui.add_space(8.0);
    if o.stacks.is_empty() {
        ui.label("No stacks yet. Create one on the left, or add a folder with a _stack.md file to the vault.");
    } else {
        stack_table(o, vault.settings.scheduler, ui);
    }
    if o.inbox_items > 0 {
        ui.label(format!("Inbox: {} items outside any stack", o.inbox_items));
    }
    problems(o, ui);
}

fn stack_table(o: &VaultOverview, default: Scheduler, ui: &mut egui::Ui) {
    Grid::new("stacks")
        .striped(true)
        .num_columns(7)
        .show(ui, |ui| {
            for head in [
                "Stack",
                "Folder",
                "Notes",
                "Cards",
                "Questions",
                "Scheduler",
                "Daily new / reviews",
            ] {
                ui.strong(head);
            }
            ui.end_row();
            for s in &o.stacks {
                ui.label(&s.title);
                ui.label(if s.folder.is_empty() {
                    "(vault root)"
                } else {
                    s.folder.as_str()
                });
                ui.label(s.notes.to_string());
                ui.label(s.cards.to_string());
                ui.label(s.questions.to_string());
                ui.label(scheduler_text(s, default));
                ui.label(format!(
                    "{} / {}",
                    limit(s.daily_new_limit),
                    limit(s.daily_review_limit)
                ));
                ui.end_row();
            }
        });
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
