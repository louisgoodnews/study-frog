//! The Backlog tab: "Up next" and "Not yet planned" with the count per status, and items that move between them by dragging
//! or by a button.

use eframe::egui::{self, CornerRadius, Frame, Id, Margin, Stroke};
use studyfrog_core::status::{STATUSES, status_label};
use studyfrog_core::uid::Uid;
use studyfrog_service::bulk::BulkEdit;

use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::stack_backlog::{backlog, counts, status_for};
use crate::state::State;
use crate::theme::palette_of;

fn counts_line(n: [usize; 5]) -> String {
    let parts: Vec<String> = STATUSES
        .iter()
        .zip(n)
        .filter(|(_, n)| *n > 0)
        .map(|(s, n)| format!("{n} {}", status_label(*s)))
        .collect();
    if parts.is_empty() {
        "empty".to_owned()
    } else {
        parts.join(", ")
    }
}

fn move_to(out: &mut Vec<Msg>, uid: Uid, up_next: bool) {
    out.push(Msg::Library(LibraryMsg::Bulk(
        crate::library_bulk::BulkMsg::MoveOne(uid, BulkEdit::Status(status_for(up_next))),
    )));
}

fn column(
    state: &State,
    (folder, title): (&str, &str),
    items: &[usize],
    up_next: bool,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let lib = &state.library;
    let pal = palette_of(ui);
    ui.label(egui::RichText::new(format!("{title} ({})", items.len())).strong());
    ui.weak(counts_line(counts(lib, items)));
    let zone = Frame::new()
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(8));
    let (_, dropped) = ui.dnd_drop_zone::<Uid, ()>(zone, |ui| {
        ui.set_min_height(80.0);
        ui.set_width(ui.available_width());
        if items.is_empty() {
            ui.weak("Nothing here.");
        }
        for i in items {
            let item = &lib.items[*i];
            let name = item.row.title.clone();
            ui.horizontal_wrapped(|ui| {
                ui.dnd_drag_source(Id::unique(("backlog", item.uid)), item.uid, |ui| {
                    ui.label(&name);
                });
                if !item.row.stacks.iter().step_by(2).any(|s| s == folder) {
                    let from = item
                        .references
                        .iter()
                        .find(|r| r.primary)
                        .map_or("Inbox", |r| r.title.as_str());
                    ui.weak(format!("(from {from})"));
                }
                if ui.link(format!("Open {name}")).clicked() {
                    out.push(Msg::OpenItem(item.uid));
                }
                let (label, target) = if up_next {
                    (format!("Move {name} to Not yet planned"), false)
                } else {
                    (format!("Move {name} to Up next"), true)
                };
                if ui.small_button(label).clicked() {
                    move_to(out, item.uid, target);
                }
            });
        }
    });
    if let Some(uid) = dropped {
        move_to(out, *uid, up_next);
    }
}

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let today = state.wall_ms.div_euclid(86_400_000);
    let b = backlog(&state.library, folder, today);
    ui.weak("Drag an item between the lists, or use its button. Up next makes an item Learning; Not yet planned puts it on hold.");
    ui.columns(2, |cols| {
        column(
            state,
            (folder, "Up next"),
            &b.up_next,
            true,
            &mut cols[0],
            out,
        );
        column(
            state,
            (folder, "Not yet planned"),
            &b.not_planned,
            false,
            &mut cols[1],
            out,
        );
    });
}
