//! Nautilus-style responsive Icon Grid view for Folders (EPIC-074).

use eframe::egui::{
    self, CornerRadius, Key, RichText, Role, ScrollArea, Stroke, TextEdit, Vec2, WidgetInfo,
};

use crate::folders_browse::{BrowseMsg, child_count, display_name, visible_dir_entries};
use crate::folders_tree::entries_from_vault;
use crate::folders_types::{FolderEntry, FoldersMsg, VisibleRow};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::folders::{icons, tree_menu};

const TILE_W: f32 = 128.0;
const TILE_H: f32 = 116.0;
const GAP: f32 = 10.0;

pub(super) fn effective_entries(state: &State) -> Vec<FolderEntry> {
    if state.folders.entries.is_empty()
        && let Some(v) = &state.vault
    {
        return entries_from_vault(v, &state.library.items);
    }
    state.folders.entries.clone()
}

/// Draw the responsive Icon Grid for the active directory.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let all = effective_entries(state);
    let items = visible_dir_entries(&all, &state.folders);

    if items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            let msg = if state.folders.show_starred_only {
                "No starred items yet."
            } else if !state.folders.filter.trim().is_empty() {
                "No items match this filter."
            } else {
                "This folder is empty. Use + Folder or + Note to add something."
            };
            ui.weak(msg);
        });
        return;
    }

    let avail = ui.available_width().max(TILE_W);
    let cols = ((avail + GAP) / (TILE_W + GAP)).floor().max(1.0) as usize;

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for chunk in items.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    for entry in chunk {
                        draw_tile(state, entry, &all, ui, out);
                    }
                });
                ui.add_space(GAP);
            }
        });
}

fn draw_tile(
    state: &State,
    entry: &FolderEntry,
    all: &[FolderEntry],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let selected = state.folders.selected.as_deref() == Some(&entry.rel_path);
    let renaming =
        state.folders.renaming.as_ref().map(|(p, _)| p.as_str()) == Some(&entry.rel_path);
    let starred = state.folders.starred.contains(&entry.rel_path);
    let label = display_name(entry);

    let fill = if selected {
        pal.surface_hover
    } else {
        egui::Color32::TRANSPARENT
    };
    let stroke = if selected {
        Stroke::new(1.5, pal.accent)
    } else {
        Stroke::new(1.0, egui::Color32::TRANSPARENT)
    };

    let frame = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(8);

    let (resp, dropped) = ui.dnd_drop_zone::<String, _>(frame, |ui| {
        ui.set_min_size(Vec2::new(TILE_W - 16.0, TILE_H - 16.0));
        ui.set_max_width(TILE_W - 16.0);
        ui.vertical_centered(|ui| {
            icons::paint_entry_icon(ui, entry, Vec2::new(56.0, 48.0));
            ui.add_space(4.0);
            if renaming {
                draw_rename_box(state, &label, ui, out);
            } else {
                let text = if starred {
                    format!("* {label}")
                } else {
                    label.clone()
                };
                ui.add(
                    egui::Label::new(RichText::new(text).color(pal.text))
                        .wrap_mode(egui::TextWrapMode::Wrap),
                );
            }
            if entry.is_folder {
                let n = child_count(all, &entry.rel_path);
                let sub = if n == 1 {
                    "1 item".to_owned()
                } else {
                    format!("{n} items")
                };
                ui.label(RichText::new(sub).small().color(pal.text_weak));
            }
        });
    });

    let tile_resp = resp
        .response
        .interact(egui::Sense::click_and_drag())
        .on_hover_text(&entry.rel_path);
    tile_resp
        .clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, label.clone()));

    if tile_resp.clicked() {
        out.push(Msg::Folders(FoldersMsg::Select(entry.rel_path.clone())));
    }
    if tile_resp.double_clicked() {
        if entry.is_folder {
            out.push(Msg::Folders(FoldersMsg::Browse(BrowseMsg::OpenDir(
                entry.rel_path.clone(),
            ))));
        } else {
            out.push(Msg::Folders(FoldersMsg::Open(entry.rel_path.clone())));
        }
    }

    let row = visible_row_for(entry, &label);
    tile_resp.context_menu(|ui| {
        tree_menu::draw_menu(&row, ui, out);
    });

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

pub(super) fn draw_rename_box(state: &State, label: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut text = state
        .folders
        .renaming
        .as_ref()
        .map_or(String::new(), |(_, t)| t.clone());
    let r = ui.add(TextEdit::singleline(&mut text).desired_width(TILE_W - 24.0));
    r.clone()
        .widget_info(|| WidgetInfo::labeled(Role::TextInput, true, format!("Rename {label}")));
    if r.changed() {
        out.push(Msg::Folders(FoldersMsg::EditRename(text)));
    }
    if ui.input(|i| i.key_pressed(Key::Enter)) {
        out.push(Msg::Folders(FoldersMsg::CommitRename));
    }
    if ui.input(|i| i.key_pressed(Key::Escape)) {
        out.push(Msg::Folders(FoldersMsg::CancelRename));
    }
}

pub(super) fn visible_row_for(entry: &FolderEntry, label: &str) -> VisibleRow {
    VisibleRow {
        rel_path: entry.rel_path.clone(),
        display_name: label.to_owned(),
        is_folder: entry.is_folder,
        is_stack: entry.is_stack,
        depth: entry.depth,
        kind: entry.kind.clone(),
        expanded: false,
    }
}
