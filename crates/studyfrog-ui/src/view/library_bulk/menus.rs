//! The bulk-edit menus of the library: set fields, tags and delete.
use super::*;

pub(super) fn menus(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.menu_button("Set status", |ui| {
        for s in STATUSES {
            if ui
                .button(status_label(s))
                .on_hover_text("Sets the status of the selected items to this.")
                .clicked()
            {
                apply(out, BulkEdit::Status(s));
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Sets the status of the selected items.");
    ui.menu_button("Set priority", |ui| {
        for p in PRIORITIES {
            if ui
                .button(priority_name(p))
                .on_hover_text("Sets the priority of the selected items to this.")
                .clicked()
            {
                apply(out, BulkEdit::Priority(p));
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Sets the priority of the selected items.");
    ui.menu_button("Set difficulty", |ui| {
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Difficult] {
            if ui
                .button(difficulty_name(d))
                .on_hover_text("Sets the difficulty of the selected items to this.")
                .clicked()
            {
                apply(out, BulkEdit::Difficulty(d));
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Sets the difficulty of the selected items.");
    let stacks_move = state.vault.iter().flat_map(|v| v.overview.stacks.iter());
    ui.menu_button("Move to stack", |ui| {
        for s in stacks_move {
            let r = ui
                .button(&s.title)
                .on_hover_text("Moves the selected items into this stack.");
            r.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, format!("Move to stack {}", s.title))
            });
            if r.clicked() {
                apply(out, BulkEdit::MoveTo(s.folder.clone()));
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Moves the selected items into a stack.");
    let stacks_add = state.vault.iter().flat_map(|v| v.overview.stacks.iter());
    ui.menu_button("Add to stack", |ui| {
        for s in stacks_add {
            let r = ui
                .button(&s.title)
                .on_hover_text("Adds the selected items to this stack.");
            r.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, format!("Add to stack {}", s.title))
            });
            if r.clicked() {
                apply(out, BulkEdit::AddTo(s.folder.clone()));
                ui.close();
            }
        }
    })
    .response
    .on_hover_text("Adds this to a stack.");
}

pub(super) fn tags(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let tag = state.library.tag_input.trim().to_owned();
    if let Some(text) = text_field(ui, "Tag", &state.library.tag_input, "", 1) {
        send(out, BulkMsg::TagInput(text));
    }
    let on = !tag.is_empty();
    if ui
        .add_enabled(on, egui::Button::new("Add tag"))
        .on_hover_text("Adds a tag to the item.")
        .clicked()
    {
        apply(out, BulkEdit::AddTag(tag.clone()));
    }
    if ui
        .add_enabled(on, egui::Button::new("Remove tag"))
        .on_hover_text("Removes this tag from the item.")
        .clicked()
    {
        apply(out, BulkEdit::RemoveTag(tag));
    }
}

pub(super) fn delete(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let n = state.library.selected.len();
    if !state.library.confirm_delete {
        if ui
            .button("Delete")
            .on_hover_text("Deletes the selected items after you confirm.")
            .clicked()
        {
            apply(out, BulkEdit::Delete);
        }
        return;
    }
    ui.label(format!(
        "Delete {n} selected item{}?",
        if n == 1 { "" } else { "s" }
    ));
    if ui
        .button("Yes, delete")
        .on_hover_text("Deletes it for good.")
        .clicked()
    {
        apply(out, BulkEdit::Delete);
    }
    if ui
        .button("Keep them")
        .on_hover_text("Keeps the items that were already there.")
        .clicked()
    {
        send(out, BulkMsg::AskDelete(false));
    }
}
