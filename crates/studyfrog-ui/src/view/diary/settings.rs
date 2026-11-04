//! Diary settings view: scale visibility, custom scales, and Home hint toggle.

use eframe::egui::{self, Role, TextEdit, WidgetInfo};
use studyfrog_core::diary::STANDARD_SCALES;

use crate::diary_msg::DiaryMsg;
use crate::diary_scales::standard_scale_info;
use crate::diary_types::CustomScaleDef;
use crate::msg::Msg;
use crate::state::State;

fn diary_msg(out: &mut Vec<Msg>, m: DiaryMsg) {
    out.push(Msg::Diary(m));
}

/// Draw the diary configuration sections in the Settings screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    home_hint_section(state, ui, out);
    ui.add_space(16.0);
    standard_scales_section(state, ui, out);
    ui.add_space(16.0);
    custom_scales_section(state, ui, out);
}

fn home_hint_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "Home screen reminder");
    crate::view::kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let mut enabled = state.diary.settings.show_home_hint;
        let cb = ui.checkbox(
            &mut enabled,
            "Show quiet 'No diary entry yet today' hint on Home screen",
        );
        cb.widget_info(|| {
            WidgetInfo::labeled(Role::CheckBox, true, "Show quiet diary hint on Home screen")
        });
        if enabled != state.diary.settings.show_home_hint {
            diary_msg(out, DiaryMsg::SetShowHomeHint(enabled));
        }
    });
}

fn standard_scales_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "Standard rating scales");
    ui.weak("Select which of the 13 standard self-assessment scales appear in the entry form.");
    ui.add_space(4.0);
    crate::view::kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for &scale_id in &STANDARD_SCALES {
            let Some(info) = standard_scale_info(scale_id) else {
                continue;
            };
            let mut is_on = state
                .diary
                .settings
                .visible_scales
                .iter()
                .any(|s| s == scale_id);
            let label = format!("{} — {}", info.name, info.question);
            let cb = ui.checkbox(&mut is_on, label);
            cb.widget_info(|| {
                WidgetInfo::labeled(
                    Role::CheckBox,
                    true,
                    format!("Toggle visibility for {}", info.name),
                )
            });
            if is_on
                != state
                    .diary
                    .settings
                    .visible_scales
                    .iter()
                    .any(|s| s == scale_id)
            {
                diary_msg(
                    out,
                    DiaryMsg::ToggleScaleVisibility(scale_id.to_owned(), is_on),
                );
            }
        }
    });
}

fn custom_scales_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    crate::view::kit::section(ui, "User-defined custom scales");
    ui.weak("Track personal factors like caffeine, environment, or physical posture.");
    ui.add_space(4.0);

    crate::view::kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if state.diary.settings.custom_scales.is_empty() {
            ui.weak("No custom scales defined.");
        } else {
            for scale in &state.diary.settings.custom_scales {
                ui.horizontal(|ui| {
                    ui.strong(&scale.name);
                    ui.label(format!("({}..={}): {}", scale.min, scale.max, scale.prompt));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let del = ui.button("Remove");
                        del.widget_info(|| {
                            WidgetInfo::labeled(
                                Role::Button,
                                true,
                                format!("Remove custom scale {}", scale.name),
                            )
                        });
                        if del.clicked() {
                            diary_msg(out, DiaryMsg::RemoveCustomScale(scale.id.clone()));
                        }
                    });
                });
                ui.separator();
            }
        }

        ui.add_space(8.0);
        add_custom_scale_form(state, ui, out);
    });
}

fn add_custom_scale_form(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(egui::RichText::new("Add custom scale").strong());
    let mut draft = state
        .diary
        .custom_scale_draft
        .clone()
        .unwrap_or_else(|| CustomScaleDef {
            id: String::new(),
            name: String::new(),
            prompt: String::new(),
            min: 1,
            max: 5,
            labels: Vec::new(),
        });

    egui::Grid::new("add-custom-scale-grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("ID:");
            let f_id = ui.add(TextEdit::singleline(&mut draft.id).hint_text("e.g. caffeine"));
            f_id.widget_info(|| {
                WidgetInfo::labeled(Role::TextInput, true, "Custom scale identifier")
            });
            ui.end_row();

            ui.label("Name:");
            let f_name =
                ui.add(TextEdit::singleline(&mut draft.name).hint_text("e.g. Caffeine intake"));
            f_name.widget_info(|| {
                WidgetInfo::labeled(Role::TextInput, true, "Custom scale display name")
            });
            ui.end_row();

            ui.label("Prompt:");
            let f_prompt =
                ui.add(TextEdit::singleline(&mut draft.prompt).hint_text("Question asked"));
            f_prompt.widget_info(|| {
                WidgetInfo::labeled(Role::TextInput, true, "Custom scale question prompt")
            });
            ui.end_row();
        });

    ui.add_space(6.0);
    let can_add = !draft.id.trim().is_empty() && !draft.name.trim().is_empty();
    ui.add_enabled_ui(can_add, |ui| {
        let add_btn = ui.button("+ Add scale");
        add_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Add custom scale"));
        if add_btn.clicked() {
            diary_msg(out, DiaryMsg::AddCustomScale(draft));
        }
    });
}
