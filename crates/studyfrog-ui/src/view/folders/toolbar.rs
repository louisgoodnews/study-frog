//! Nautilus/Thunar-style header toolbar for the Folders view (EPIC-074).

use eframe::egui::{self, ComboBox, RichText, Role, TextEdit, WidgetInfo};

use crate::folders_browse::{BrowseMsg, FoldersViewMode, breadcrumbs};
use crate::folders_types::{FoldersMsg, SortOrder};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

fn sort_label(sort: SortOrder) -> &'static str {
    match sort {
        SortOrder::NameAsc => "Name (A to Z)",
        SortOrder::NameDesc => "Name (Z to A)",
        SortOrder::MtimeDesc => "Modified (newest)",
        SortOrder::MtimeAsc => "Modified (oldest)",
        SortOrder::KindAsc => "Type (A to Z)",
        SortOrder::KindDesc => "Type (Z to A)",
        SortOrder::SizeDesc => "Size (largest)",
        SortOrder::SizeAsc => "Size (smallest)",
    }
}

/// Draw the Nautilus-style navigation, breadcrumb, view-switcher and action toolbar.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let f = &state.folders;

    // Row 1: Navigation, Breadcrumb path bar, inline filter, and View Switcher
    ui.horizontal_wrapped(|ui| {
        draw_nav_buttons(f, ui, out);
        ui.separator();
        draw_breadcrumbs(&f.current_dir, pal.text_weak, ui, out);
        ui.separator();

        let mut query = f.filter.clone();
        let edit = ui.add(
            TextEdit::singleline(&mut query)
                .desired_width(140.0)
                .hint_text("Filter folder…"),
        );
        edit.clone()
            .widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Filter folder"));
        if edit.changed() {
            out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::SetFilter(
                query,
            ))));
        }

        ui.separator();
        draw_view_switcher(f.view_mode, ui, out);
    });

    ui.add_space(6.0);

    // Row 2: Create actions, Sort order, Folders-first, Starred filter, and Tree Expand/Collapse
    ui.horizontal_wrapped(|ui| {
        let note_btn = ui.button("+ Note").on_hover_text("Starts a new note.");
        note_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "New note"));
        if note_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::StartNewNote));
        }

        let folder_btn = ui.button("+ Folder").on_hover_text("Makes a new folder.");
        folder_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "New folder"));
        if folder_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::StartNewFolder));
        }

        ui.separator();
        draw_sort_combo(f.sort, ui, out);

        let ff_btn = ui
            .selectable_label(!f.folders_mixed, "Folders first")
            .on_hover_text("Keep folders before files.");
        ff_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Folders first"));
        if ff_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::Browse(
                BrowseMsg::ToggleFoldersFirst,
            )));
        }

        let star_btn = ui
            .selectable_label(f.show_starred_only, "* Starred")
            .on_hover_text("Show only starred items.");
        star_btn
            .clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Starred filter"));
        if star_btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::Browse(
                BrowseMsg::ToggleStarredOnly,
            )));
        }

        if f.view_mode == FoldersViewMode::Tree {
            ui.separator();
            let expand_btn = ui.button("Expand").on_hover_text("Shows the hidden part.");
            expand_btn
                .clone()
                .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Expand all"));
            if expand_btn.clicked() {
                out.push(Msg::Folders(FoldersMsg::ExpandAll));
            }

            let collapse_btn = ui.button("Collapse").on_hover_text("Hides the detail.");
            collapse_btn
                .clone()
                .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Collapse all"));
            if collapse_btn.clicked() {
                out.push(Msg::Folders(FoldersMsg::CollapseAll));
            }
        }
    });
}

fn draw_nav_buttons(f: &crate::folders_types::FoldersState, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let back = ui
        .add_enabled(!f.nav_back.is_empty(), egui::Button::new("<"))
        .on_hover_text("Go back to previous folder.");
    back.clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, !f.nav_back.is_empty(), "Folder back"));
    if back.clicked() {
        out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::NavBack)));
    }

    let fwd = ui
        .add_enabled(!f.nav_forward.is_empty(), egui::Button::new(">"))
        .on_hover_text("Go forward to next folder.");
    fwd.clone().widget_info(|| {
        WidgetInfo::labeled(Role::Button, !f.nav_forward.is_empty(), "Folder forward")
    });
    if fwd.clicked() {
        out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::NavForward)));
    }

    let up = ui
        .add_enabled(!f.current_dir.is_empty(), egui::Button::new("^"))
        .on_hover_text("Go up to parent folder.");
    up.clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, !f.current_dir.is_empty(), "Folder up"));
    if up.clicked() {
        out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::NavUp)));
    }
}

fn draw_breadcrumbs(current_dir: &str, weak: egui::Color32, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let crumbs = breadcrumbs(current_dir);
    for (idx, (label, path)) in crumbs.iter().enumerate() {
        if idx > 0 {
            ui.label(RichText::new("/").color(weak));
        }
        let active = idx + 1 == crumbs.len();
        let btn = ui
            .selectable_label(active, label)
            .on_hover_text(if path.is_empty() {
                "Go to vault root."
            } else {
                path.as_str()
            });
        btn.clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Path {label}")));
        if btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::OpenDir(
                path.clone(),
            ))));
        }
    }
}

fn draw_view_switcher(mode: FoldersViewMode, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    for (m, label, name) in [
        (FoldersViewMode::Grid, "Grid", "Grid view"),
        (FoldersViewMode::List, "List", "List view"),
        (FoldersViewMode::Tree, "Tree", "Tree view"),
    ] {
        let btn = ui
            .selectable_label(mode == m, label)
            .on_hover_text(format!("Switch to {name}."));
        btn.clone()
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
        if btn.clicked() {
            out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::SetViewMode(m))));
        }
    }
}

fn draw_sort_combo(sort: SortOrder, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let combo = ComboBox::from_id_salt("folders-sort")
        .selected_text(sort_label(sort))
        .show_ui(ui, |ui| {
            for order in [
                SortOrder::NameAsc,
                SortOrder::NameDesc,
                SortOrder::MtimeDesc,
                SortOrder::MtimeAsc,
                SortOrder::KindAsc,
                SortOrder::KindDesc,
                SortOrder::SizeDesc,
                SortOrder::SizeAsc,
            ] {
                if ui
                    .selectable_label(sort == order, sort_label(order))
                    .clicked()
                {
                    out.push(Msg::Folders(FoldersMsg::SetSort(order)));
                }
            }
        });
    combo
        .response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Sort order"));
}
