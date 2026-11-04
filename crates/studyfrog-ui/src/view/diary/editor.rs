//! Entry editor form with ratings, links, minutes, and Markdown reflections.

use eframe::egui::{self, Role, TextEdit, WidgetInfo};
use studyfrog_core::civil::format_date;

use super::editor_ratings::ratings_section;
use crate::diary_msg::DiaryMsg;
use crate::diary_types::DiaryEditDraft;
use crate::msg::Msg;
use crate::state::State;

fn diary_msg(out: &mut Vec<Msg>, m: DiaryMsg) {
    out.push(Msg::Diary(m));
}

/// Draw the diary editor form for `state.diary.active_entry`.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(draft) = &state.diary.active_entry else {
        return;
    };

    ui.horizontal(|ui| {
        let title = if draft.uid.is_some() {
            format!("Edit Diary: {}", format_date(draft.date))
        } else {
            format!("New Diary Entry: {}", format_date(draft.date))
        };
        ui.heading(title);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let cancel_btn = ui.button("Cancel");
            cancel_btn.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, "Cancel editing diary entry")
            });
            if cancel_btn.clicked() {
                diary_msg(out, DiaryMsg::CloseEditor);
            }

            let save_btn = crate::view::kit::primary_button(ui, "Save entry");
            save_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Save diary entry"));
            if save_btn.clicked() {
                diary_msg(out, DiaryMsg::SaveDraft);
            }
        });
    });

    ui.separator();
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            metadata_section(draft, ui, out);
            ui.add_space(16.0);
            ratings_section(state, draft, ui, out);
            ui.add_space(16.0);
            body_section(draft, ui, out);
            ui.add_space(16.0);
            bottom_actions(draft, ui, out);
        });
}

fn metadata_section(draft: &DiaryEditDraft, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Grid::new("diary-draft-meta")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Title:");
                let mut title = draft.title.clone();
                let def_hint = format!("Diary: {}", format_date(draft.date));
                let field = ui.add(
                    TextEdit::singleline(&mut title)
                        .hint_text(def_hint)
                        .desired_width(ui.available_width()),
                );
                field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Entry title"));
                if title != draft.title {
                    diary_msg(out, DiaryMsg::UpdateDraftTitle(title));
                }
                ui.end_row();

                ui.label("Study minutes:");
                let mut mins_str = draft.study_minutes.map_or(String::new(), |m| m.to_string());
                let field = ui.add(
                    TextEdit::singleline(&mut mins_str)
                        .hint_text("e.g. 45")
                        .desired_width(120.0),
                );
                field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Study minutes"));
                if mins_str != draft.study_minutes.map_or(String::new(), |m| m.to_string()) {
                    let parsed = mins_str.trim().parse::<u32>().ok();
                    diary_msg(out, DiaryMsg::UpdateDraftMinutes(parsed));
                }
                ui.end_row();

                ui.label("Linked stacks:");
                let mut stacks = draft.stacks.clone();
                let field = ui.add(
                    TextEdit::singleline(&mut stacks)
                        .hint_text("e.g. biochem/metabolism, anatomy")
                        .desired_width(ui.available_width()),
                );
                field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Linked stacks"));
                if stacks != draft.stacks {
                    diary_msg(out, DiaryMsg::UpdateDraftStacks(stacks));
                }
                ui.end_row();

                ui.label("Tags:");
                let mut tags = draft.tags.clone();
                let field = ui.add(
                    TextEdit::singleline(&mut tags)
                        .hint_text("e.g. exam-prep, evening")
                        .desired_width(ui.available_width()),
                );
                field.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Entry tags"));
                if tags != draft.tags {
                    diary_msg(out, DiaryMsg::UpdateDraftTags(tags));
                }
                ui.end_row();
            });
    });
}

fn body_section(draft: &DiaryEditDraft, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "Reflections & takeaways (Markdown)");
    let mut body = draft.content.clone();
    let edit = ui.add(
        TextEdit::multiline(&mut body)
            .desired_rows(12)
            .desired_width(ui.available_width())
            .hint_text("What went well? What was challenging? What will you do next time?"),
    );
    edit.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Diary reflection body"));
    if body != draft.content {
        diary_msg(out, DiaryMsg::UpdateDraftContent(body));
    }
}

fn bottom_actions(draft: &DiaryEditDraft, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        let save_btn = crate::view::kit::primary_button(ui, "Save entry");
        save_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Save diary entry"));
        if save_btn.clicked() {
            diary_msg(out, DiaryMsg::SaveDraft);
        }

        let cancel_btn = ui.button("Cancel");
        cancel_btn
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Cancel editing diary entry"));
        if cancel_btn.clicked() {
            diary_msg(out, DiaryMsg::CloseEditor);
        }

        if let Some(uid) = draft.uid {
            ui.add_space(16.0);
            let open_btn = ui.button("Open in Markdown editor");
            open_btn.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, "Open in full Markdown editor")
            });
            if open_btn.clicked() {
                out.push(Msg::OpenItem(uid));
            }

            let del_btn = ui.button("Delete entry");
            del_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Delete diary entry"));
            if del_btn.clicked() {
                diary_msg(out, DiaryMsg::DeleteEntry(uid));
            }
        }
    });
}
