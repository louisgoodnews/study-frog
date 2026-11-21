//! The parts of a board card: its buttons, its editor and the stack picker.
use super::*;

pub(super) fn card_buttons(card: &Card, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
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
        if ui
            .small_button(label)
            .on_hover_text("Moves the card to this column.")
            .clicked()
        {
            send(out, msg);
        }
    }
}

pub(super) fn card_editor(state: &State, e: &CardEdit, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
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
            if kit::primary_button(ui, "Save card")
                .on_hover_text("Saves this card.")
                .clicked()
            {
                send(out, BoardMsg::SaveCard);
            }
            if ui
                .button("Close card")
                .on_hover_text("Closes this card.")
                .clicked()
            {
                send(out, BoardMsg::CloseCard);
            }
        });
    });
}

pub(super) fn stack_picker(
    state: &State,
    current: Option<Uid>,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
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
