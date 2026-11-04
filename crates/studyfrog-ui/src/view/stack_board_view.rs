//! The Board tab: the stack's items in a column for each status. An item moves by dragging it to another column or with its
//! menu, which writes the status into its file.

use eframe::egui::{self, CornerRadius, Frame, Id, Margin, Stroke};
use studyfrog_core::status::{STATUSES, Status, status_label};
use studyfrog_core::uid::Uid;
use studyfrog_service::bulk::BulkEdit;

use crate::library_bulk::BulkMsg;
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

fn move_to(out: &mut Vec<Msg>, uid: Uid, status: Status) {
    out.push(Msg::Library(LibraryMsg::Bulk(BulkMsg::MoveOne(
        uid,
        BulkEdit::Status(status),
    ))));
}

fn card(
    uid: Uid,
    title: &str,
    status: Status,
    from: Option<&str>,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    ui.dnd_drag_source(Id::unique(("board-card", uid)), uid, |ui| {
        ui.label(title);
    });
    if let Some(primary) = from {
        ui.weak(format!("(from {primary})"));
    }
    ui.horizontal(|ui| {
        if ui.link(format!("Open {title}")).clicked() {
            out.push(Msg::OpenItem(uid));
        }
        ui.menu_button(format!("Move {title}"), |ui| {
            for s in STATUSES.iter().filter(|s| **s != status) {
                if ui.button(status_label(*s)).clicked() {
                    move_to(out, uid, *s);
                    ui.close();
                }
            }
        });
    });
    ui.separator();
}

fn column(state: &State, folder: &str, status: Status, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let members: Vec<_> = state
        .library
        .items
        .iter()
        .filter(|i| {
            i.row.status == status
                && (i.row.stacks.iter().step_by(2).any(|c| c == folder)
                    || i.references
                        .iter()
                        .any(|r| r.folder.as_deref() == Some(folder)))
        })
        .collect();
    ui.label(egui::RichText::new(format!("{} ({})", status_label(status), members.len())).strong());
    let zone = Frame::new()
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(6));
    let (_, dropped) = ui.dnd_drop_zone::<Uid, ()>(zone, |ui| {
        ui.set_min_height(100.0);
        ui.set_width(ui.available_width());
        for i in &members {
            let from = if !i.row.stacks.iter().step_by(2).any(|c| c == folder) {
                i.references
                    .iter()
                    .find(|r| r.primary)
                    .map(|r| r.title.as_str())
            } else {
                None
            };
            card(i.uid, &i.row.title, status, from, ui, out);
        }
    });
    if let Some(uid) = dropped {
        move_to(out, *uid, status);
    }
}

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.weak("Drag an item to another column, or use its Move menu.");
    ui.columns(STATUSES.len(), |cols| {
        for (col, status) in cols.iter_mut().zip(STATUSES) {
            column(state, folder, status, col, out);
        }
    });
}
