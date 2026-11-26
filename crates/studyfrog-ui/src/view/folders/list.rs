//! Nautilus/Thunar-style Detailed List view for Folders (EPIC-074).

use eframe::egui::{self, CornerRadius, RichText, Role, ScrollArea, Vec2, WidgetInfo};

use crate::folders_browse::{
    BrowseMsg, SortColumn, child_count, display_name, kind_label, visible_dir_entries,
};
use crate::folders_types::{FolderEntry, FoldersMsg, SortOrder};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::folders::grid::{draw_rename_box, effective_entries, visible_row_for};
use crate::view::folders::status::{format_modified, format_size};
use crate::view::folders::{icons, tree_menu};

fn col_arrow(sort: SortOrder, col: SortColumn) -> &'static str {
    match (col, sort) {
        (SortColumn::Name, SortOrder::NameAsc)
        | (SortColumn::Kind, SortOrder::KindAsc)
        | (SortColumn::Size, SortOrder::SizeAsc)
        | (SortColumn::Modified, SortOrder::MtimeAsc) => " ^",
        (SortColumn::Name, SortOrder::NameDesc)
        | (SortColumn::Kind, SortOrder::KindDesc)
        | (SortColumn::Size, SortOrder::SizeDesc)
        | (SortColumn::Modified, SortOrder::MtimeDesc) => " v",
        _ => "",
    }
}

/// Draw the sortable multi-column Detailed List view.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let all = effective_entries(state);
    let items = visible_dir_entries(&all, &state.folders);
    let avail = ui.available_width().max(420.0);
    let kind_w = 96.0;
    let size_w = 96.0;
    let mod_w = 110.0;
    let star_w = 34.0;
    let name_w = (avail - kind_w - size_w - mod_w - star_w - 40.0).max(140.0);

    draw_header(state.folders.sort, [name_w, kind_w, size_w, mod_w], ui, out);
    ui.separator();

    if items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.weak("No entries to show in this folder.");
        });
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for entry in &items {
                draw_row(state, entry, &all, [name_w, kind_w, size_w, mod_w], ui, out);
            }
        });
}

fn draw_header(sort: SortOrder, widths: [f32; 4], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    ui.horizontal(|ui| {
        for (idx, (title, col)) in [
            ("Name", SortColumn::Name),
            ("Type", SortColumn::Kind),
            ("Size", SortColumn::Size),
            ("Modified", SortColumn::Modified),
        ]
        .into_iter()
        .enumerate()
        {
            let label = format!("{title}{}", col_arrow(sort, col));
            let header = egui::Button::new(RichText::new(label).small().color(pal.text_weak)) // hover: from the caller
                .frame(false);
            let btn = ui
                .add_sized(Vec2::new(widths[idx], 24.0), header)
                .on_hover_text(format!("Sort by {title}."));
            btn.clone().widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, format!("Sort by {title}"))
            });
            if btn.clicked() {
                out.push(Msg::Folders(FoldersMsg::Browse(
                    BrowseMsg::ToggleSortColumn(col),
                )));
            }
        }
    });
}

fn draw_row(
    state: &State,
    entry: &FolderEntry,
    all: &[FolderEntry],
    widths: [f32; 4],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let now_ms = state.home.as_ref().map_or(0, |h| h.now_ms);
    let pal = palette_of(ui);
    let selected = state.folders.selected.as_deref() == Some(&entry.rel_path);
    let renaming =
        state.folders.renaming.as_ref().map(|(p, _)| p.as_str()) == Some(&entry.rel_path);
    let starred = state.folders.starred.contains(&entry.rel_path);
    let label = display_name(entry);

    let frame = egui::Frame::new()
        .fill(if selected {
            pal.surface_hover
        } else {
            egui::Color32::TRANSPARENT
        })
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(6, 4));

    let (resp, dropped) = ui.dnd_drop_zone::<String, _>(frame, |ui| {
        ui.horizontal(|ui| {
            // Name column
            ui.allocate_ui_with_layout(
                Vec2::new(widths[0], 24.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    icons::paint_entry_icon(ui, entry, Vec2::new(20.0, 18.0));
                    if renaming {
                        draw_rename_box(state, &label, ui, out);
                    } else {
                        let name_resp = ui.selectable_label(selected, &label);
                        name_resp
                            .clone()
                            .widget_info(|| WidgetInfo::labeled(Role::Button, true, label.clone()));
                        if name_resp.clicked() {
                            out.push(Msg::Folders(FoldersMsg::Select(entry.rel_path.clone())));
                        }
                        if name_resp.double_clicked() {
                            open_or_enter(entry, out);
                        }
                        let row = visible_row_for(entry, &label);
                        name_resp.context_menu(|ui| tree_menu::draw_menu(&row, ui, out));
                    }
                },
            );

            // Type column
            ui.add_sized(
                Vec2::new(widths[1], 24.0),
                egui::Label::new(RichText::new(kind_label(entry)).color(pal.text_weak)),
            );

            // Size / Items column
            let size_text = if entry.is_folder {
                let n = child_count(all, &entry.rel_path);
                if n == 1 {
                    "1 item".to_owned()
                } else {
                    format!("{n} items")
                }
            } else {
                format_size(entry.size_bytes)
            };
            ui.add_sized(
                Vec2::new(widths[2], 24.0),
                egui::Label::new(RichText::new(size_text).color(pal.text_weak)),
            );

            // Modified column
            ui.add_sized(
                Vec2::new(widths[3], 24.0),
                egui::Label::new(
                    RichText::new(format_modified(entry.mtime_ns, now_ms)).color(pal.text_weak),
                ),
            );

            // Star column
            let star_glyph = if starred { "*" } else { "-" };
            let star_btn = ui
                .button(RichText::new(star_glyph).color(if starred {
                    pal.warning
                } else {
                    pal.text_weak
                }))
                .on_hover_text("Star or unstar this item.");
            star_btn
                .clone()
                .widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Star {label}")));
            if star_btn.clicked() {
                out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::ToggleStar(
                    entry.rel_path.clone(),
                ))));
            }
        });
    });

    if resp.response.clicked() {
        out.push(Msg::Folders(FoldersMsg::Select(entry.rel_path.clone())));
    }
    if resp.response.double_clicked() {
        open_or_enter(entry, out);
    }

    if let Some(src_path) = dropped
        && entry.is_folder
        && src_path.as_str() != entry.rel_path.as_str()
    {
        out.push(Msg::Folders(FoldersMsg::AskMove((*src_path).clone())));
        out.push(Msg::Folders(FoldersMsg::SetMoveTarget(
            entry.rel_path.clone(),
        )));
        out.push(Msg::Folders(FoldersMsg::CommitMove));
    }
}

fn open_or_enter(entry: &FolderEntry, out: &mut Vec<Msg>) {
    if entry.is_folder {
        out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::OpenDir(
            entry.rel_path.clone(),
        ))));
    } else {
        out.push(Msg::Folders(FoldersMsg::Open(entry.rel_path.clone())));
    }
}
