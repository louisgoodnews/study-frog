//! What the window shows before a vault is open. Choosing the vault is not the first thing asked:
//! an in-app notification (see `toast`) invites the user to set it up, and this page has the same
//! door. The greeting is on the gradient panel; the actions and each recent vault are redesign
//! cards (EPIC-073, STORY-0232). The actions and messages are unchanged.
use eframe::egui::{self, RichText};

use crate::backup_ui::BackupMsg;
use crate::modal::ModalMsg;
use crate::msg::{Msg, MsgVault};
use crate::state::State;
use crate::theme::MIN_TARGET;
use crate::theme::style::Style;
use crate::view::{gradient_panel, home_hero, kit};

/// The height of the greeting panel, in points (EPIC-073, STORY-0232).
const HEADLINE_HEIGHT: f32 = 150.0;

/// A greeting on the gradient panel, the actions in one card, and each recent vault in a card.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    studyfrog_log::log_debug!("ui", "welcome shown");
    let _timer = studyfrog_debug::timed("ui", "welcome draw");
    let size = egui::vec2(520.0, ui.available_height());
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.add_space(16.0);
        gradient_panel::panel(ui, HEADLINE_HEIGHT, greeting);
        ui.add_space(12.0);
        actions(state, ui, out);
        recent(state, ui, out);
    });
}

/// The title and the one-paragraph description, on the gradient.
fn greeting(ui: &mut egui::Ui, s: Style) {
    ui.label(
        RichText::new("Welcome to StudyFrog")
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong()
            .color(s.gradient_text),
    );
    ui.add(
        egui::Label::new(
            RichText::new("Write notes in Markdown, turn them into flashcards, and rehearse them with spaced repetition. Everything stays in a folder on this computer.")
                .color(s.gradient_text),
        )
        .wrap(),
    );
}

/// The open and restore actions, with the backup and the profiles, in one card.
fn actions(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        if state.opening {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Opening and indexing the vault…");
            });
        } else if kit::primary_button(ui, "Open or create a vault…")
            .on_hover_text("Opens a vault, or creates one in a new folder.")
            .clicked()
        {
            out.push(Msg::Modal(ModalMsg::OpenVault));
        }
        if !state.opening
            && kit::quiet_button(ui, "Restore from a backup…")
                .on_hover_text("Restores the vault from a backup file you choose.")
                .clicked()
        {
            out.push(Msg::Backup(BackupMsg::Choose));
        }
        if let Some(c) = &state.backup.chosen {
            ui.add_space(8.0);
            crate::view::backup::chosen(state, c, ui, out);
        }
        if state.profiles.list.names.len() > 1 {
            ui.add_space(12.0);
            super::profiles::draw(state, ui, out);
        }
    });
}

/// The recent vaults, one card each; a click opens that vault.
fn recent(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.app.recent.is_empty() {
        return;
    }
    ui.add_space(12.0);
    kit::caption(ui, "Recent vaults");
    for path in &state.app.recent {
        home_hero::card(ui, |ui| {
            ui.set_width(ui.available_width());
            let size = egui::vec2(ui.available_width(), MIN_TARGET);
            let button = egui::Button::new(path.as_str()).min_size(size);
            if ui
                .add_enabled(!state.opening, button)
                .on_hover_text("Opens this recent vault.")
                .clicked()
            {
                out.push(Msg::Vault(MsgVault::OpenRecent(path.clone())));
            }
        });
    }
}
