//! The Sub-stacks tab: the stacks inside this one as a tree that folds, each with status, progress, due date and familiarity.

use eframe::egui::{self, Id};
use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_core::status::status_label;
use studyfrog_service::stack_summary::SubStack;

use crate::msg::Msg;
use crate::state::{Selection, State};

fn fold_id(folder: &str) -> Id {
    Id::unique(("stack-fold", folder))
}

fn line(sub: &SubStack) -> String {
    let due = sub.due_day.map_or("no due date".to_owned(), |d| {
        format!("due {}", format_date(date_from_days(d)))
    });
    let fam = sub
        .percent
        .map_or("no history".to_owned(), |p| format!("familiarity {p} %"));
    format!(
        "{} - {} of {} done - {due} - {fam}",
        status_label(sub.status),
        sub.done,
        sub.total
    )
}

fn row(tree: &[SubStack], i: usize, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let sub = &tree[i];
    let has_children = tree.get(i + 1).is_some_and(|n| n.depth > sub.depth);
    ui.horizontal(|ui| {
        ui.add_space(16.0 * (sub.depth - 1) as f32);
        if has_children {
            let id = fold_id(&sub.folder);
            let open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(true);
            let name = format!("{} {}", if open { "Fold" } else { "Unfold" }, sub.title);
            if ui.small_button(name).clicked() {
                ui.data_mut(|d| d.insert_temp(id, !open));
            }
        }
        if ui.link(&sub.title).clicked() {
            out.push(Msg::Select(Selection::Stack(sub.folder.clone())));
        }
        ui.weak(line(sub));
    });
}

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let held = state
        .library
        .stack_view
        .summary
        .as_ref()
        .filter(|(f, _, _)| f == folder);
    let Some((_, _, s)) = held else {
        ui.label("Reading the sub-stacks…");
        return;
    };
    if s.tree.is_empty() {
        ui.weak("No stacks inside this one. Create one with New item and choose Stack.");
        return;
    }
    let mut hidden_below: Option<usize> = None;
    for i in 0..s.tree.len() {
        let depth = s.tree[i].depth;
        if hidden_below.is_some_and(|d| depth > d) {
            continue;
        }
        hidden_below = None;
        row(&s.tree, i, ui, out);
        let id = fold_id(&s.tree[i].folder);
        if !ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(true) {
            hidden_below = Some(depth);
        }
    }
}
