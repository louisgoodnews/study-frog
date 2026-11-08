//! Individual tree row rendering for Folders view.

use eframe::egui::{self, Key, RichText, Role, Stroke, TextEdit, WidgetInfo};

use crate::folders_types::{EntryKind, FoldersMsg, VisibleRow};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

/// Draw one row of the tree.
pub fn draw_row(state: &State, row: &VisibleRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let selected = state.folders.selected.as_deref() == Some(&row.rel_path);
    let renaming = state.folders.renaming.as_ref().map(|(p, _)| p.as_str()) == Some(&row.rel_path);

    let (_, dropped) = ui.dnd_drop_zone::<String, ()>(egui::Frame::new(), |ui| {
        ui.horizontal(|ui| {
            draw_indents(row.depth, ui);

            if row.is_folder {
                draw_chevron(row, ui, out);
            } else {
                ui.add_space(16.0);
            }

            draw_badge(row, ui);

            if renaming {
                draw_rename_field(state, row, ui, out);
            } else {
                draw_label(row, selected, ui, out);
            }
        });
    });

    if let Some(src_path) = dropped
        && row.is_folder
        && src_path.as_str() != row.rel_path.as_str()
    {
        out.push(Msg::Folders(FoldersMsg::AskMove((*src_path).clone())));
        out.push(Msg::Folders(FoldersMsg::SetMoveTarget(
            row.rel_path.clone(),
        )));
        out.push(Msg::Folders(FoldersMsg::CommitMove));
    }
}

fn draw_indents(depth: usize, ui: &mut egui::Ui) {
    for _ in 0..depth {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 20.0), egui::Sense::hover());
        let x = rect.left() + 8.0;
        let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
        ui.painter().vline(x, rect.y_range(), stroke);
    }
}

fn draw_chevron(row: &VisibleRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let icon = if row.expanded { "v" } else { ">" };
    let btn = ui.small_button(icon);
    btn.clone().widget_info(|| {
        WidgetInfo::labeled(Role::Button, true, format!("Toggle {}", row.display_name))
    });
    if btn.clicked() {
        out.push(Msg::Folders(FoldersMsg::ToggleExpand(row.rel_path.clone())));
    }
}

fn draw_badge(row: &VisibleRow, ui: &mut egui::Ui) {
    let palette = palette_of(ui);
    if row.is_folder {
        if row.is_stack {
            ui.label(RichText::new("[S]").color(palette.accent).strong());
        } else {
            ui.label(RichText::new("[F]").color(palette.text_weak));
        }
    } else {
        match &row.kind {
            EntryKind::Pdf => {
                ui.label(
                    RichText::new("[PDF]")
                        .size(crate::typo::size(ui, crate::typo::Tier::Caption))
                        .color(palette.accent),
                );
            }
            EntryKind::Image => {
                ui.label(
                    RichText::new("[IMG]")
                        .size(crate::typo::size(ui, crate::typo::Tier::Caption))
                        .color(palette.text_weak),
                );
            }
            EntryKind::Other(ext) => {
                ui.label(
                    RichText::new(format!("[{ext}]"))
                        .size(crate::typo::size(ui, crate::typo::Tier::Caption))
                        .color(palette.text_weak),
                );
            }
            EntryKind::Markdown | EntryKind::Folder => {}
        }
    }
}

fn draw_rename_field(state: &State, row: &VisibleRow, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut text = state
        .folders
        .renaming
        .as_ref()
        .map_or(String::new(), |(_, t)| t.clone());
    let r = ui.add(TextEdit::singleline(&mut text));
    r.clone().widget_info(|| {
        WidgetInfo::labeled(
            Role::TextInput,
            true,
            format!("Rename {}", row.display_name),
        )
    });
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

fn draw_label(row: &VisibleRow, selected: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let resp = ui.selectable_label(selected, &row.display_name);
    resp.clone()
        .widget_info(|| WidgetInfo::labeled(Role::Button, true, row.display_name.clone()));

    if resp.clicked() {
        out.push(Msg::Folders(FoldersMsg::Select(row.rel_path.clone())));
    }
    if resp.double_clicked() {
        out.push(Msg::Folders(FoldersMsg::Open(row.rel_path.clone())));
    }

    resp.context_menu(|ui| {
        super::tree_menu::draw_menu(row, ui, out);
    });
}
