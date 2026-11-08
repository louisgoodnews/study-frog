//! Post-rehearsal diary prompt view component. Pure: no I/O, emits messages.
//! See Section 5 of `docs/specs/diary.md`.

use eframe::egui::{self, Key, RichText, Role, WidgetInfo};
use studyfrog_core::civil::format_date;
use studyfrog_core::diary::{RatingValue, rating_standard_score};
use studyfrog_md::ulid::format_ulid;

use crate::diary_msg::DiaryMsg;
use crate::diary_scales::{scale_level_label, scale_level_tooltip, standard_scale_info};
use crate::msg::Msg;
use crate::run_state::RunView;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

const DEFAULT_PRIMARY: [&str; 3] = ["focus", "energy", "satisfaction"];

/// Draw the post-rehearsal diary prompt card on the run summary screen.
pub fn draw(state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let prompt_opt = run.diary_prompt.as_ref();
    if let Some(p) = prompt_opt {
        if p.skipped {
            return;
        }
        if p.saved {
            draw_saved_card(ui, out);
            return;
        }
    }
    let data = run
        .diary_prompt
        .clone()
        .unwrap_or_else(|| crate::diary_prompt::prompt_new(state, run));

    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Log your session in the diary?")
                    .size(crate::typo::size(ui, crate::typo::Tier::Body))
                    .strong(),
            );
            let mut parts = vec![
                format!("Date: {}", format_date(data.date)),
                format!("Duration: {} min", data.study_minutes),
                format!("Run: {}", format_ulid(data.run_uid)),
            ];
            if !data.stacks.is_empty() {
                parts.push(format!("Stack: {}", data.stacks.join(", ")));
            }
            parts.push(format!("{} items reviewed", data.items.len()));
            ui.weak(parts.join("  ·  "));
        });
        ui.add_space(2.0);
        draw_scales(state, &data, ui, out);
        ui.add_space(2.0);
        draw_reflection_input(&data, ui, out);
        ui.add_space(4.0);
        draw_actions(ui, out);
    });
}

fn draw_saved_card(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Session logged in diary.")
                    .color(p.success)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn = ui.button("Dismiss");
                btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Dismiss"));
                if btn.clicked() {
                    out.push(Msg::Diary(DiaryMsg::PromptSkip));
                }
            });
        });
    });
}

fn draw_scales(
    state: &State,
    data: &crate::diary_prompt::RunDiaryPrompt,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let configured = &state.diary.settings.primary_scales;
    let fallback: Vec<String> = DEFAULT_PRIMARY.iter().map(|&s| s.to_owned()).collect();
    let scales = if configured.is_empty() {
        &fallback
    } else {
        configured
    };
    for scale_id in scales {
        if let Some(info) = standard_scale_info(scale_id) {
            draw_scale_row(info, data, ui, out);
        }
    }
}

fn draw_scale_row(
    info: &crate::diary_scales::StandardScaleInfo,
    data: &crate::diary_prompt::RunDiaryPrompt,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let current = data.ratings.get(info.id).and_then(rating_standard_score);
    let p = palette_of(ui);
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{}:", info.name)).strong());
        for level in 1..=5 {
            let sel = current == Some(level);
            let desc = scale_level_tooltip(info.id, level);
            let a11y = format!("Rate {} level {level}", info.name);
            let btn = ui
                .selectable_label(sel, format!("{level}"))
                .on_hover_text(&desc);
            btn.widget_info(|| WidgetInfo::labeled(Role::RadioButton, true, &a11y));
            if btn.clicked() {
                let next = if sel {
                    None
                } else {
                    Some(RatingValue::Int(i64::from(level)))
                };
                out.push(Msg::Diary(DiaryMsg::PromptRating(info.id.to_owned(), next)));
            }
        }
        if let Some(lvl) = current {
            if let Some((lbl, _)) = scale_level_label(info.id, lvl) {
                ui.label(RichText::new(format!("— {lbl}")).color(p.accent).strong());
            }
            let clr = ui.button("Clear");
            clr.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Clear rating"));
            if clr.clicked() {
                out.push(Msg::Diary(DiaryMsg::PromptRating(info.id.to_owned(), None)));
            }
        }
    });
}

fn draw_reflection_input(
    data: &crate::diary_prompt::RunDiaryPrompt,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let mut note = data.note.clone();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Reflection:").strong());
        let res = ui.add(
            egui::TextEdit::singleline(&mut note)
                .hint_text("Quick reflections or takeaways...")
                .desired_width(ui.available_width() - 8.0),
        );
        res.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Reflection"));
        if res.changed() {
            out.push(Msg::Diary(DiaryMsg::PromptNote(note)));
        }
        if res.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            out.push(Msg::Diary(DiaryMsg::PromptSubmit));
        }
    });
}

fn draw_actions(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        let save_btn = kit::primary_button(ui, "Save entry (Enter)");
        save_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Save entry (Enter)"));
        if save_btn.clicked() {
            out.push(Msg::Diary(DiaryMsg::PromptSubmit));
        }
        ui.add_space(8.0);
        let more_btn = ui.button("More details...");
        more_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "More details..."));
        if more_btn.clicked() {
            out.push(Msg::Diary(DiaryMsg::PromptExpand));
        }
        ui.add_space(8.0);
        let skip_btn = ui.button("Skip (Esc)");
        skip_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Skip (Esc)"));
        if skip_btn.clicked() {
            out.push(Msg::Diary(DiaryMsg::PromptSkip));
        }
    });
}
