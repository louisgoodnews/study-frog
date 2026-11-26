//! The Settings tabs as a rail of buttons (STORY-0227): the selected tab carries the rail highlight.
use eframe::egui::{self, RichText};

use crate::app_settings_ui::{SHOWN, SettingsTab, SettingsTabMsg, tab_label};
use crate::msg::Msg;
use crate::theme::palette_of;
use crate::theme::style::style_of;

/// One button per tab, in order; a click selects that tab.
pub fn buttons(ui: &mut egui::Ui, current: SettingsTab, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    let s = style_of(&p);
    ui.horizontal_wrapped(|ui| {
        for tab in SHOWN {
            let selected = tab == current;
            let text = if selected { p.accent } else { p.text };
            let fill = if selected { s.rail_active } else { s.rail };
            let label = RichText::new(tab_label(tab)).color(text);
            if ui
                .add(egui::Button::new(label).fill(fill))
                .on_hover_text(format!("Shows the {} settings.", tab_label(tab)))
                .clicked()
            {
                out.push(Msg::SettingsTab(SettingsTabMsg::Select(tab)));
            }
        }
    });
}
