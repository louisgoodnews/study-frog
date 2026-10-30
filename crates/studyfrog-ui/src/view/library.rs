//! The stack overview.

use eframe::egui::{self, Grid};
use studyfrog_core::scheduler::{Scheduler, scheduler_name};
use studyfrog_store::overview::{StackRow, VaultOverview};

use crate::state::State;

fn limit(n: Option<u32>) -> String {
    n.map_or_else(|| "–".to_owned(), |v| v.to_string())
}

fn scheduler_text(row: &StackRow, default: Scheduler) -> String {
    match row.scheduler {
        Some(s) => scheduler_name(s).to_owned(),
        None => format!("{} (default)", scheduler_name(default)),
    }
}

/// Title, counts and a table of stacks; the inbox and any problems below it.
pub fn draw(state: &State, ui: &mut egui::Ui) {
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
        ui.label("No stacks yet. Add a folder with a _stack.md file to the vault.");
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
