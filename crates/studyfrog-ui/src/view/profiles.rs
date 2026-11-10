//! Overhauled profile switcher and manager (STORY-0138): avatar badge, clean card layouts, safe delete flow.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText};
use studyfrog_config::profiles::DEFAULT_PROFILE;

use crate::msg::Msg;
use crate::profile_ui::ProfileMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: ProfileMsg) {
    out.push(Msg::Profile(m));
}

fn avatar_badge(ui: &mut egui::Ui, name: &str, bg: Color32, fg: Color32) {
    let initial = name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.label(RichText::new(initial).color(fg).strong());
        });
}

/// The profiles of this install.
pub(super) fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.profiles;
    let pal = palette_of(ui);

    kit::card(ui, |ui| {
        ui.horizontal(|ui| {
            avatar_badge(ui, &p.list.current, pal.accent, pal.on_accent);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Profile: {}", p.list.current)).strong());
                    kit::pill(ui, "Active", pal.accent);
                });
                ui.weak("Current active workspace");
            });
        });
    });

    let others: Vec<&String> = p
        .list
        .names
        .iter()
        .filter(|n| **n != p.list.current)
        .collect();

    if !others.is_empty() {
        ui.add_space(4.0);
        ui.weak("Switch profile");
        for name in others {
            kit::card(ui, |ui| {
                ui.horizontal(|ui| {
                    avatar_badge(ui, name, pal.surface_hover, pal.text);
                    let switch_btn =
                        egui::Button::new(RichText::new(format!("Switch to {name}")).strong())
                            .min_size(egui::vec2(120.0, 24.0));
                    if ui.add(switch_btn).clicked() {
                        send(out, ProfileMsg::Switch(name.clone()));
                    }

                    if *name != DEFAULT_PROFILE {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if p.confirm_delete.as_deref() == Some(name.as_str()) {
                                if ui.small_button("Keep").clicked() {
                                    send(out, ProfileMsg::CancelDelete);
                                }
                                if ui
                                    .button(
                                        RichText::new(format!("Really delete {name}"))
                                            .color(pal.danger),
                                    )
                                    .clicked()
                                {
                                    send(out, ProfileMsg::ConfirmDelete);
                                }
                            } else if ui.small_button(format!("Delete {name}")).clicked() {
                                send(out, ProfileMsg::AskDelete(name.clone()));
                            }
                        });
                    }
                });
            });
            ui.add_space(2.0);
        }
    }

    if p.confirm_delete.is_some() {
        ui.add_space(2.0);
        ui.label(
            RichText::new(
                "This removes the profile's settings and API keys, not its vault folder.",
            )
            .small()
            .color(pal.text_weak),
        );
    }

    ui.add_space(6.0);
    kit::card(ui, |ui| {
        ui.label(RichText::new("Add new profile").strong());
        ui.add_space(2.0);
        if let Some(t) = text_field(ui, "New profile", &p.draft, "Profile name…", 1) {
            send(out, ProfileMsg::Draft(t));
        }
        ui.add_space(4.0);
        if ui
            .add(
                egui::Button::new(RichText::new("Create profile").strong())
                    .fill(pal.accent)
                    .min_size(egui::vec2(ui.available_width(), 26.0)),
            )
            .clicked()
        {
            send(out, ProfileMsg::Create);
        }
    });
}
