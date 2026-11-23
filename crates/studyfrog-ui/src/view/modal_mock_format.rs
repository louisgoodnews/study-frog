//! Interactive modal for configuring mock exam item formats and mix (STORY-0207).

use eframe::egui::{self, Align, Layout, RichText, Role, WidgetInfo};
use studyfrog_rehearsal::exam_format::{
    ALL_EXAM_FORMATS, ExamItemFormat, FormatPlan, MAX_EXAM_QUESTIONS, MixMode, format_index,
    format_label, plan_count, resolve_format_plan, resolved_mix_total,
};

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::setup::mock_exam::MockExamMsg;
use crate::setup::mock_exam::format_modal::{FormatModalMsg, MockExamFormatModal};
use crate::theme::palette_of;
use crate::view::icons::{Glyph, icon_button};
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: FormatModalMsg) {
    out.push(Msg::MockExam(MockExamMsg::FormatModal(m)));
}

/// Render the mock exam item format selection modal.
pub fn draw(modal: &MockExamFormatModal, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new("Configure mock exam")
                    .strong()
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading)),
            );
            ui.label(RichText::new(&modal.title).color(pal.text_weak));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                out.push(Msg::Modal(ModalMsg::Close));
            }
        });
    });
    ui.add_space(8.0);
    mode_and_total_row(modal, ui, out);
    ui.add_space(8.0);
    let plan = resolve_format_plan(&modal.mix).ok();
    kit::card(ui, |ui| {
        for fmt in ALL_EXAM_FORMATS {
            format_row(modal, fmt, plan.as_ref(), ui, out);
        }
    });
    ui.add_space(8.0);
    summary_and_error(modal, plan.as_ref(), ui);
    ui.add_space(10.0);
    footer_buttons(modal, ui, out);
}

fn mode_and_total_row(modal: &MockExamFormatModal, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Mix by:");
        let weights_btn = ui
            .add(egui::Button::selectable(
                modal.mix.mode == MixMode::Weights,
                "Proportions",
            ))
            .on_hover_text(
                "Set relative weights and let StudyFrog scale them to the total question count.",
            );
        weights_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Mix by proportions"));
        if weights_btn.clicked() {
            send(out, FormatModalMsg::SetMode(MixMode::Weights));
        }
        let counts_btn = ui
            .add(egui::Button::selectable(
                modal.mix.mode == MixMode::Counts,
                "Exact counts",
            ))
            .on_hover_text("Set the exact number of questions for each enabled format.");
        counts_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Mix by exact counts"));
        if counts_btn.clicked() {
            send(out, FormatModalMsg::SetMode(MixMode::Counts));
        }
        ui.add_space(12.0);
        if modal.mix.mode == MixMode::Weights {
            ui.label("Total questions:");
            let dec = ui
                .small_button("-")
                .on_hover_text("Decrease total questions by 1.");
            dec.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Decrease total questions"));
            if dec.clicked() {
                send(out, FormatModalMsg::StepTotal(-1));
            }
            let mut total = modal.mix.total_questions;
            let drag = ui
                .add(egui::DragValue::new(&mut total).range(1..=MAX_EXAM_QUESTIONS))
                .on_hover_text("Total questions across all selected formats (1 to 50).");
            drag.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Total questions"));
            if drag.changed() {
                send(out, FormatModalMsg::SetTotal(total));
            }
            let inc = ui
                .small_button("+")
                .on_hover_text("Increase total questions by 1.");
            inc.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Increase total questions"));
            if inc.clicked() {
                send(out, FormatModalMsg::StepTotal(1));
            }
        } else {
            let total = resolved_mix_total(&modal.mix);
            ui.label(RichText::new(format!("Total: {total} questions")).strong());
        }
    });
}

fn format_row(
    modal: &MockExamFormatModal,
    fmt: ExamItemFormat,
    plan: Option<&FormatPlan>,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let alloc = modal.mix.formats[format_index(fmt)];
    let label = format_label(fmt);
    ui.horizontal(|ui| {
        let mut enabled = alloc.enabled;
        let chk = ui.checkbox(&mut enabled, label);
        chk.widget_info(|| WidgetInfo::labeled(Role::CheckBox, true, format!("Include {label}")));
        if chk.changed() {
            send(out, FormatModalMsg::ToggleFormat(fmt, enabled));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let resolved = plan.map_or(0, |p| plan_count(p, fmt));
            kit::pill(
                ui,
                &format!("{resolved} q"),
                if alloc.enabled {
                    pal.accent
                } else {
                    pal.text_weak
                },
            );
            ui.add_enabled_ui(alloc.enabled, |ui| {
                let inc = ui
                    .small_button("+")
                    .on_hover_text(format!("Increase {label} value."));
                inc.widget_info(|| {
                    WidgetInfo::labeled(Role::Button, true, format!("Increase {label}"))
                });
                if inc.clicked() {
                    send(out, FormatModalMsg::StepValue(fmt, 1));
                }
                let mut val = alloc.value;
                let unit = if modal.mix.mode == MixMode::Weights {
                    "weight"
                } else {
                    "count"
                };
                let drag = ui
                    .add(egui::DragValue::new(&mut val).range(0..=MAX_EXAM_QUESTIONS))
                    .on_hover_text(format!("{label} {unit}."));
                drag.widget_info(|| {
                    WidgetInfo::labeled(Role::TextInput, true, format!("{label} {unit}"))
                });
                if drag.changed() {
                    send(out, FormatModalMsg::SetValue(fmt, val));
                }
                let dec = ui
                    .small_button("-")
                    .on_hover_text(format!("Decrease {label} value."));
                dec.widget_info(|| {
                    WidgetInfo::labeled(Role::Button, true, format!("Decrease {label}"))
                });
                if dec.clicked() {
                    send(out, FormatModalMsg::StepValue(fmt, -1));
                }
            });
        });
    });
}

fn summary_and_error(modal: &MockExamFormatModal, plan: Option<&FormatPlan>, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    if let Some(p) = plan {
        let parts: Vec<String> = ALL_EXAM_FORMATS
            .iter()
            .filter_map(|&fmt| {
                let n = plan_count(p, fmt);
                (n > 0).then(|| format!("{} ({n})", format_label(fmt)))
            })
            .collect();
        let total = resolved_mix_total(&modal.mix);
        ui.weak(format!("Summary: {} · {total} questions", parts.join(", ")));
    }
    if let Some(err) = &modal.error {
        ui.colored_label(pal.danger, err);
    }
}

fn footer_buttons(modal: &MockExamFormatModal, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        if kit::quiet_button(ui, "Reset defaults")
            .on_hover_text("Restore the default mix across all six item formats.")
            .clicked()
        {
            send(out, FormatModalMsg::ResetDefaults);
        }
        if ui
            .button("Cancel")
            .on_hover_text("Close without generating an exam.")
            .clicked()
        {
            out.push(Msg::Modal(ModalMsg::Close));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let valid = modal.error.is_none();
            ui.add_enabled_ui(valid, |ui| {
                if kit::primary_button(ui, "Generate exam")
                    .on_hover_text("Generate the mock exam with the selected item formats.")
                    .clicked()
                {
                    send(out, FormatModalMsg::Confirm);
                }
            });
        });
    });
}
