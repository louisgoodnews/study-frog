//! Rating row components for the diary editor form.

use eframe::egui::{self, RichText, Role, WidgetInfo};
use studyfrog_core::diary::{RatingValue, rating_standard_score};

use crate::diary_msg::DiaryMsg;
use crate::diary_scales::{scale_level_label, scale_level_tooltip, standard_scale_info};
use crate::diary_types::DiaryEditDraft;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

fn diary_msg(out: &mut Vec<Msg>, m: DiaryMsg) {
    out.push(Msg::Diary(m));
}

/// Draw the ratings section inside the editor form.
pub fn ratings_section(
    state: &State,
    draft: &DiaryEditDraft,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    crate::view::kit::section(ui, "Self-assessment ratings");
    crate::view::kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for &scale_id in &studyfrog_core::diary::STANDARD_SCALES {
            if !state
                .diary
                .settings
                .visible_scales
                .iter()
                .any(|s| s == scale_id)
            {
                continue;
            }
            if let Some(info) = standard_scale_info(scale_id) {
                standard_scale_row(info, draft, ui, out);
                ui.add_space(8.0);
            }
        }
        for custom in &state.diary.settings.custom_scales {
            custom_scale_row(custom, draft, ui, out);
            ui.add_space(8.0);
        }
    });
}

fn standard_scale_row(
    info: &crate::diary_scales::StandardScaleInfo,
    draft: &DiaryEditDraft,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let current = draft.ratings.get(info.id).and_then(rating_standard_score);
    let p = palette_of(ui);

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(info.name).strong());
        ui.label(RichText::new(format!("({})", info.question)).color(p.text_weak));
    });

    ui.horizontal(|ui| {
        for level in 1..=5 {
            let sel = current == Some(level);
            let desc = scale_level_tooltip(info.id, level);
            let a11y = format!("Rate {} level {desc}", info.name);
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
                diary_msg(out, DiaryMsg::UpdateDraftRating(info.id.to_owned(), next));
            }
        }

        if let Some(lvl) = current {
            if let Some((lbl, _)) = scale_level_label(info.id, lvl) {
                ui.label(RichText::new(format!("— {lbl}")).color(p.accent).strong());
            }
            let clear_btn = ui.button("Clear");
            clear_btn.widget_info(|| {
                WidgetInfo::labeled(
                    Role::Button,
                    true,
                    format!("Clear rating for {}", info.name),
                )
            });
            if clear_btn.clicked() {
                diary_msg(out, DiaryMsg::UpdateDraftRating(info.id.to_owned(), None));
            }
        }
    });
}

fn custom_scale_row(
    scale: &crate::diary_types::CustomScaleDef,
    draft: &DiaryEditDraft,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let current = draft
        .ratings
        .get(&scale.id)
        .and_then(studyfrog_core::diary::rating_int);
    let p = palette_of(ui);

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&scale.name).strong());
        ui.label(RichText::new(format!("({})", scale.prompt)).color(p.text_weak));
    });

    ui.horizontal(|ui| {
        for val in scale.min..=scale.max {
            let sel = current == Some(val);
            let idx = (val - scale.min) as usize;
            let label = scale
                .labels
                .get(idx)
                .map_or_else(|| format!("{val}"), |l| format!("{val}: {l}"));
            let a11y = format!("Rate {} with {label}", scale.name);
            let btn = ui.selectable_label(sel, &label);
            btn.widget_info(|| WidgetInfo::labeled(Role::RadioButton, true, &a11y));
            if btn.clicked() {
                let next = if sel {
                    None
                } else {
                    Some(RatingValue::Int(val))
                };
                diary_msg(out, DiaryMsg::UpdateDraftRating(scale.id.clone(), next));
            }
        }
    });
}
