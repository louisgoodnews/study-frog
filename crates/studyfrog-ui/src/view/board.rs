//! The kanban board: columns side by side with WIP counts, draggable cards, and button
//! alternatives to every drag so the board works from the keyboard and with a screen reader.

use eframe::egui::{self, CornerRadius, Frame, Id, Margin, RichText, Stroke};
use studyfrog_board::model::{Card, Column, over_limit};
use studyfrog_core::uid::Uid;

use crate::kanban::{BoardMsg, CardEdit};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{palette_of, tint};
use crate::view::kit;
use crate::view::widgets::text_field;

const MIN_COLUMN_HEIGHT: f32 = 320.0;

fn send(out: &mut Vec<Msg>, m: BoardMsg) {
    out.push(Msg::Board(m));
}

/// Heading, controls, the columns, and the card editor when a card is open.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(
        RichText::new("Board")
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong(),
    );
    ui.add_space(4.0);
    let k = &state.kanban;
    let Some(board) = &k.board else {
        ui.label("Loading the board…");
        return;
    };
    ui.horizontal(|ui| {
        let mut force = k.force;
        if ui
            .checkbox(&mut force, "Allow exceeding WIP limits")
            .changed()
        {
            send(out, BoardMsg::Force(force));
        }
    });
    ui.horizontal(|ui| {
        let label = ui.label("New card title");
        let mut title = k.new_title.clone();
        // The button must stay on screen: the field takes what is left of the row.
        let room = (ui.available_width() - 110.0).max(120.0);
        let field = egui::TextEdit::singleline(&mut title)
            .hint_text("added to the first column")
            .desired_width(room);
        ui.add(field).labelled_by(label.id);
        if title != k.new_title {
            send(out, BoardMsg::NewTitle(title));
        }
        if kit::primary_button(ui, "Add card").clicked() {
            send(out, BoardMsg::Add);
        }
    });
    if let Some(edit) = &k.edit {
        card_editor(state, edit, ui, out);
    }
    if board.columns.is_empty() {
        return;
    }
    // Every column gets an equal share of the width there is (less the scroll bar's room), so none runs off the edge.
    let n = board.columns.len() as f32;
    let gap = ui.spacing().item_spacing.x;
    let width = ((ui.available_width() - 16.0 - gap * (n - 1.0)) / n).max(160.0);
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                for col in &board.columns {
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_width(width);
                            column(state, col, ui, out);
                        },
                    );
                }
            });
        });
}

fn count_text(col: &Column) -> String {
    let n = col.cards.len();
    match col.wip_limit {
        Some(limit) => format!("{} {n}/{limit}", col.name),
        None => format!("{} {n}", col.name),
    }
}

fn column(state: &State, col: &Column, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let over = state
        .kanban
        .board
        .as_ref()
        .is_some_and(|b| over_limit(b).contains(&col.id));
    let mut title = RichText::new(count_text(col)).strong();
    if over {
        title = title.color(ui.visuals().error_fg_color);
    }
    ui.label(title);
    if over {
        ui.colored_label(ui.visuals().error_fg_color, "Over the WIP limit");
    }
    let mut rects = Vec::new();
    let pal = palette_of(ui);
    let dragging = egui::DragAndDrop::has_payload_of_type::<Uid>(ui.ctx());
    let fill = if dragging {
        tint(pal.accent, 24)
    } else {
        pal.panel
    };
    let zone = Frame::new()
        .fill(fill)
        .stroke(Stroke::new(
            1.0,
            if dragging { pal.accent } else { pal.border },
        ))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(8));
    let (_, dropped) = ui.dnd_drop_zone::<Uid, ()>(zone, |ui| {
        ui.set_min_height(MIN_COLUMN_HEIGHT);
        ui.set_width(ui.available_width());
        for card in &col.cards {
            rects.push(card_view(state, card, ui, out).rect);
        }
    });
    if let Some(card) = dropped {
        let y = ui
            .input(|i| i.pointer.interact_pos())
            .map_or(f32::MAX, |p| p.y);
        let index = rects.iter().filter(|r| r.center().y < y).count();
        send(
            out,
            BoardMsg::Move {
                card: *card,
                to: col.id,
                index,
            },
        );
    }
}

fn stack_name(state: &State, uid: Option<Uid>) -> Option<String> {
    let v = state.vault.as_ref()?;
    let uid = uid?;
    v.overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.title.clone())
}

fn card_view(state: &State, card: &Card, ui: &mut egui::Ui, out: &mut Vec<Msg>) -> egui::Response {
    let pal = palette_of(ui);
    Frame::new()
        .fill(pal.surface)
        .stroke(Stroke::new(1.0, pal.border))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let face = ui.dnd_drag_source(Id::unique(("card", card.id)), card.id, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(&card.title).strong());
                let extra = [
                    stack_name(state, card.stack),
                    card.points.map(|p| format!("{p} pts")),
                ];
                let line: Vec<String> = extra.into_iter().flatten().collect();
                if !line.is_empty() {
                    ui.label(RichText::new(line.join(" · ")).small().color(pal.text_weak));
                }
            });
            if face.response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            ui.horizontal_wrapped(|ui| card_buttons(card, ui, out));
        })
        .response
}

fn card_buttons(card: &Card, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let t = &card.title;
    let id = card.id;
    let buttons = [
        (format!("Pull {t}"), BoardMsg::Pull(id)),
        (format!("Push {t}"), BoardMsg::Push(id)),
        (format!("Move {t} up"), BoardMsg::Shift(id, -1)),
        (format!("Move {t} down"), BoardMsg::Shift(id, 1)),
        (format!("Edit {t}"), BoardMsg::Open(id)),
        (format!("Delete {t}"), BoardMsg::Remove(id)),
    ];
    for (label, msg) in buttons {
        if ui.small_button(label).clicked() {
            send(out, msg);
        }
    }
}

fn card_editor(state: &State, e: &CardEdit, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::group(ui, "Edit card", |ui| {
        if let Some(t) = text_field(ui, "Card title", &e.title, "", 1) {
            send(out, BoardMsg::EditTitle(t));
        }
        if let Some(t) = text_field(ui, "Card notes", &e.notes, "", 3) {
            send(out, BoardMsg::EditNotes(t));
        }
        if let Some(t) = text_field(ui, "Estimate (points)", &e.points, "", 1) {
            send(out, BoardMsg::EditPoints(t));
        }
        stack_picker(state, e.stack, ui, out);
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Save card").clicked() {
                send(out, BoardMsg::SaveCard);
            }
            if ui.button("Close card").clicked() {
                send(out, BoardMsg::CloseCard);
            }
        });
    });
}

fn stack_picker(state: &State, current: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stacks = state
        .vault
        .as_ref()
        .map(|v| v.overview.stacks.as_slice())
        .unwrap_or_default();
    let name = stack_name(state, current).unwrap_or_else(|| "No stack".to_owned());
    egui::ComboBox::from_label("Linked stack")
        .selected_text(name)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "No stack").clicked() {
                send(out, BoardMsg::EditStack(None));
            }
            for s in stacks {
                if ui
                    .selectable_label(current == Some(s.uid), &s.title)
                    .clicked()
                {
                    send(out, BoardMsg::EditStack(Some(s.uid)));
                }
            }
        });
}
