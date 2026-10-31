//! The profile part of the profile menu: who is using the app, switching, creating and deleting.

use eframe::egui::{self, RichText};
use studyfrog_config::profiles::DEFAULT_PROFILE;

use crate::msg::Msg;
use crate::profile_ui::ProfileMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: ProfileMsg) {
    out.push(Msg::Profile(m));
}

/// The profiles of this install.
pub(super) fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.profiles;
    ui.label(RichText::new(format!("Profile: {}", p.list.current)).strong());
    for name in p.list.names.iter().filter(|n| **n != p.list.current) {
        ui.horizontal(|ui| {
            if ui.button(format!("Switch to {name}")).clicked() {
                send(out, ProfileMsg::Switch(name.clone()));
            }
            if name == DEFAULT_PROFILE {
                return;
            }
            if p.confirm_delete.as_deref() == Some(name.as_str()) {
                if ui.button(format!("Really delete {name}")).clicked() {
                    send(out, ProfileMsg::ConfirmDelete);
                }
                if ui.small_button("Keep").clicked() {
                    send(out, ProfileMsg::CancelDelete);
                }
            } else if ui.small_button(format!("Delete {name}")).clicked() {
                send(out, ProfileMsg::AskDelete(name.clone()));
            }
        });
    }
    if p.confirm_delete.is_some() {
        ui.label(
            RichText::new(
                "This removes the profile's settings and API keys, not its vault folder.",
            )
            .small()
            .color(palette_of(ui).text_weak),
        );
    }
    if let Some(t) = text_field(ui, "New profile", &p.draft, "name", 1) {
        send(out, ProfileMsg::Draft(t));
    }
    if ui.button("Create profile").clicked() {
        send(out, ProfileMsg::Create);
    }
}
