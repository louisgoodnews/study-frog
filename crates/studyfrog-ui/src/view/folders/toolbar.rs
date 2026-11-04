//! Toolbar for the Folders view.

use eframe::egui::{self, ComboBox, Role, WidgetInfo};

use crate::folders_types::{FoldersMsg, SortOrder};
use crate::msg::Msg;
use crate::state::State;

fn sort_label(sort: SortOrder) -> &'static str {
    match sort {
        SortOrder::NameAsc => "File name (A to Z)",
        SortOrder::NameDesc => "File name (Z to A)",
        SortOrder::MtimeDesc => "Modified time (newest first)",
        SortOrder::MtimeAsc => "Modified time (oldest first)",
    }
}

/// Draw the Folders toolbar.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        let note_btn = ui.button("+ Note");
        note_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "New note"));
        if note_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::StartNewNote));
        }

        let folder_btn = ui.button("+ Folder");
        folder_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "New folder"));
        if folder_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::StartNewFolder));
        }

        ui.separator();

        let combo = ComboBox::from_id_salt("folders-sort")
            .selected_text(sort_label(state.folders.sort))
            .show_ui(ui, |ui| {
                for (order, label) in [
                    (SortOrder::NameAsc, "File name (A to Z)"),
                    (SortOrder::NameDesc, "File name (Z to A)"),
                    (SortOrder::MtimeDesc, "Modified time (newest first)"),
                    (SortOrder::MtimeAsc, "Modified time (oldest first)"),
                ] {
                    if ui
                        .selectable_label(state.folders.sort == order, label)
                        .clicked()
                    {
                        out.push(Msg::Folders(FoldersMsg::SetSort(order)));
                    }
                }
            });
        combo
            .response
            .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Sort order"));

        ui.separator();

        let expand_btn = ui.button("Expand");
        expand_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Expand all"));
        if expand_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::ExpandAll));
        }

        let collapse_btn = ui.button("Collapse");
        collapse_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Collapse all"));
        if collapse_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::CollapseAll));
        }
    });
}
