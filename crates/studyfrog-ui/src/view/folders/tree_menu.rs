//! Context menu actions for folders and files.

use eframe::egui::{self, Role, WidgetInfo};

use crate::folders_types::{FoldersMsg, VisibleRow};
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::{Screen, Selection};

/// Draw the context menu for a row.
pub fn draw_menu(row: &VisibleRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if row.is_folder {
        draw_folder_menu(row, ui, out);
    } else {
        draw_file_menu(row, ui, out);
    }
}

fn draw_folder_menu(row: &VisibleRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let new_note = ui.button("New note");
    new_note
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "New note"));
    if new_note.clicked() {
        out.push(Msg::Folders(FoldersMsg::Select(row.rel_path.clone())));
        out.push(Msg::Folders(FoldersMsg::StartNewNote));
        ui.close();
    }

    let new_folder = ui.button("New folder");
    new_folder
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "New folder"));
    if new_folder.clicked() {
        out.push(Msg::Folders(FoldersMsg::Select(row.rel_path.clone())));
        out.push(Msg::Folders(FoldersMsg::StartNewFolder));
        ui.close();
    }

    ui.separator();

    if row.is_stack {
        let unmark = ui.button("Convert to normal folder");
        unmark
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Convert to normal folder"));
        if unmark.clicked() {
            out.push(Msg::Modal(ModalMsg::AskUnmark(row.rel_path.clone())));
            ui.close();
        }
    } else {
        let mark = ui.button("Mark as stack folder");
        mark.clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Mark as stack folder"));
        if mark.clicked() {
            out.push(Msg::Modal(ModalMsg::MarkAsStack(row.rel_path.clone())));
            ui.close();
        }
    }

    let show_items = ui.button("Show in Items");
    show_items
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Show in Items"));
    if show_items.clicked() {
        out.push(Msg::Navigate(Screen::Library));
        out.push(Msg::Select(Selection::Stack(row.rel_path.clone())));
        ui.close();
    }

    ui.separator();

    let rename = ui.button("Rename");
    rename
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Rename folder"));
    if rename.clicked() {
        out.push(Msg::Folders(FoldersMsg::StartRename(row.rel_path.clone())));
        ui.close();
    }

    let move_to = ui.button("Move to…");
    move_to
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Move folder to"));
    if move_to.clicked() {
        out.push(Msg::Folders(FoldersMsg::AskMove(row.rel_path.clone())));
        ui.close();
    }

    let delete = ui.button("Delete");
    delete
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Delete folder"));
    if delete.clicked() {
        out.push(Msg::Folders(FoldersMsg::AskDelete(row.rel_path.clone())));
        ui.close();
    }
}

fn draw_file_menu(row: &VisibleRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let open = ui.button("Open");
    open.clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Open file"));
    if open.clicked() {
        out.push(Msg::Folders(FoldersMsg::Open(row.rel_path.clone())));
        ui.close();
    }

    let rename = ui.button("Rename");
    rename
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Rename file"));
    if rename.clicked() {
        out.push(Msg::Folders(FoldersMsg::StartRename(row.rel_path.clone())));
        ui.close();
    }

    let move_to = ui.button("Move to…");
    move_to
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Move file to"));
    if move_to.clicked() {
        out.push(Msg::Folders(FoldersMsg::AskMove(row.rel_path.clone())));
        ui.close();
    }

    let delete = ui.button("Delete");
    delete
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Delete file"));
    if delete.clicked() {
        out.push(Msg::Folders(FoldersMsg::AskDelete(row.rel_path.clone())));
        ui.close();
    }
}
