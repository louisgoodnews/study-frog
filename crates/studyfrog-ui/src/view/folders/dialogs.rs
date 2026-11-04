//! Dialogs and popovers for the Folders view.

use eframe::egui::{self, Key, Role, TextEdit, WidgetInfo, Window};

use crate::folders_types::FoldersMsg;
use crate::msg::Msg;
use crate::state::State;

/// Draw popovers and confirmation dialogs.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    draw_new_folder(state, ui, out);
    draw_confirm_move(state, ui, out);
    draw_confirm_delete(state, ui, out);
    draw_move_picker(state, ui, out);
}

fn draw_new_folder(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(form) = &state.folders.new_folder else {
        return;
    };
    Window::new("New Folder")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label(format!("Create in: /{}", form.parent));
            let mut name = form.name.clone();
            let mut is_stack = form.is_stack;

            let field = ui.add(TextEdit::singleline(&mut name));
            field
                .clone()
                .widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Folder name"));
            let chk = ui.checkbox(&mut is_stack, "Mark as stack");
            chk.clone()
                .widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, "Mark as stack"));

            if field.changed() || chk.changed() {
                out.push(Msg::Folders(FoldersMsg::EditNewFolder(name, is_stack)));
            }

            ui.horizontal(|ui| {
                let create_btn = ui.button("Create");
                create_btn
                    .clone()
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Create folder"));
                if create_btn.clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                    out.push(Msg::Folders(FoldersMsg::CommitNewFolder));
                }

                let cancel_btn = ui.button("Cancel");
                cancel_btn.clone().widget_info(|| {
                    WidgetInfo::labeled(Role::Button, true, "Cancel folder creation")
                });
                if cancel_btn.clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                    out.push(Msg::Folders(FoldersMsg::CancelNewFolder));
                }
            });
        });
}

fn draw_confirm_move(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(plan) = &state.folders.confirm_move else {
        return;
    };
    Window::new("Confirm Move Across Stacks")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            let msg = format!(
                "Moving {} items to '{}'. This will change their primary stack.",
                plan.items_changing_primary_stack, plan.destination_stack_title
            );
            ui.label(msg);

            ui.horizontal(|ui| {
                let confirm_btn = ui.button("Confirm Move");
                confirm_btn
                    .clone()
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Confirm move"));
                if confirm_btn.clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                    out.push(Msg::Folders(FoldersMsg::ConfirmMove));
                }

                let cancel_btn = ui.button("Cancel");
                cancel_btn
                    .clone()
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Cancel move"));
                if cancel_btn.clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                    out.push(Msg::Folders(FoldersMsg::CancelMove));
                }
            });
        });
}

fn draw_confirm_delete(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(plan) = &state.folders.confirm_delete else {
        return;
    };
    Window::new("Confirm Deletion")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            let msg = format!(
                "Delete '{}'? This will delete {} files.",
                plan.target, plan.files_count
            );
            ui.label(msg);

            ui.horizontal(|ui| {
                let delete_btn = ui.button("Delete");
                delete_btn
                    .clone()
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Confirm delete"));
                if delete_btn.clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                    out.push(Msg::Folders(FoldersMsg::ConfirmDelete));
                }

                let cancel_btn = ui.button("Cancel");
                cancel_btn
                    .clone()
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Cancel delete"));
                if cancel_btn.clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                    out.push(Msg::Folders(FoldersMsg::CancelDelete));
                }
            });
        });
}

fn draw_move_picker(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some((src, current_dst)) = &state.folders.move_target else {
        return;
    };
    Window::new(format!("Move \"{src}\""))
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.label("Select destination folder:");

            let root_sel = ui.selectable_label(current_dst.is_empty(), "/ (root)");
            root_sel
                .clone()
                .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Destination root folder"));
            if root_sel.clicked() {
                out.push(Msg::Folders(FoldersMsg::SetMoveTarget(String::new())));
            }

            for entry in &state.folders.entries {
                if entry.is_folder
                    && entry.rel_path != *src
                    && !entry.rel_path.starts_with(&format!("{src}/"))
                {
                    let label = format!("/{}", entry.rel_path);
                    let sel = ui.selectable_label(current_dst == &entry.rel_path, &label);
                    sel.clone().widget_info(|| {
                        WidgetInfo::labeled(
                            Role::Button,
                            true,
                            format!("Destination folder {label}"),
                        )
                    });
                    if sel.clicked() {
                        out.push(Msg::Folders(FoldersMsg::SetMoveTarget(
                            entry.rel_path.clone(),
                        )));
                    }
                }
            }

            ui.horizontal(|ui| {
                let move_btn = ui.button("Move");
                move_btn
                    .clone()
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Move entry"));
                if move_btn.clicked() {
                    out.push(Msg::Folders(FoldersMsg::CommitMove));
                }

                let cancel_btn = ui.button("Cancel");
                cancel_btn.clone().widget_info(|| {
                    WidgetInfo::labeled(Role::Button, true, "Cancel move destination")
                });
                if cancel_btn.clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                    out.push(Msg::Folders(FoldersMsg::CancelMove));
                }
            });
        });
}
