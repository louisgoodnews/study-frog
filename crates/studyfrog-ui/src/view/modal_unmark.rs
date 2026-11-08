//! The confirmation modal for turning a stack into a normal folder (ADR 0011, TASK-0217).

use eframe::egui::{self, Align, Layout, RichText, Role, WidgetInfo};
use studyfrog_service::stack_mark::UnmarkPlan;

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::theme::palette_of;
use crate::view::icons::{Glyph, icon_button};
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: ModalMsg) {
    out.push(Msg::Modal(m));
}

/// Draw the confirmation dialog.
pub fn draw(plan: &UnmarkPlan, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    egui::Frame::window(ui.style()).show(ui, |ui| {
        ui.set_width(460.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Make \"{}\" a normal folder?", plan.title))
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                    send(out, ModalMsg::Close);
                }
            });
        });

        ui.add_space(8.0);
        ui.label(RichText::new(
            "Turning this stack into a normal folder removes the stack mark (_stack.md). All files inside the folder will remain.",
        ).color(p.text_weak));

        ui.add_space(10.0);
        kit::section(ui, "What will change");
        kit::card(ui, |ui| {
            ui.set_width(ui.available_width());

            // Settings
            if plan.has_settings {
                ui.label("• Stack settings in _stack.toml will be removed");
            } else {
                ui.label("• Default vault settings will be applied to items");
            }

            // Status
            if let Some(s) = plan.status {
                ui.label(format!("• Stack status ({s:?}) will be removed"));
            }

            // Goals
            if plan.goals_count > 0 {
                ui.label(format!(
                    "• {} goal(s) defined in the stack file will be removed",
                    plan.goals_count
                ));
            }

            // Subjects
            if !plan.subjects.is_empty() {
                ui.label(format!(
                    "• Subjects ({}) will be removed from the stack",
                    plan.subjects.join(", ")
                ));
            }

            // Sub-stacks
            if !plan.sub_stacks.is_empty() {
                ui.label(format!(
                    "• {} sub-stack(s) will remain as independent stacks",
                    plan.sub_stacks.len()
                ));
            }

            // Referencing items
            if !plan.referencing_items.is_empty() {
                ui.label(format!(
                    "• {} item(s) referencing this stack will have the reference removed",
                    plan.referencing_items.len()
                ));
            }

            // Items
            ui.label(format!(
                "• {} item(s) in this folder will belong to {}",
                plan.items_count, plan.target_title
            ));
        });

        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let confirm = ui.button(
                    RichText::new("Make normal folder").color(p.warning),
                );
                confirm.widget_info(|| {
                    WidgetInfo::labeled(
                        Role::Button,
                        true,
                        format!("Confirm making {} a normal folder", plan.title),
                    )
                });
                if confirm.clicked() {
                    send(out, ModalMsg::ConfirmUnmark(plan.folder.clone()));
                }

                let cancel = ui.button("Cancel");
                cancel.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Cancel"));
                if cancel.clicked() {
                    send(out, ModalMsg::Close);
                }
            });
        });
    });
}
