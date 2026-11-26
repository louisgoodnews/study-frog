//! The Settings tabs as a rail of buttons (STORY-0227).
use eframe::egui;

use crate::app_settings_ui::{SHOWN, SettingsTab, SettingsTabMsg, tab_label};
use crate::msg::Msg;
use crate::view::rail;

/// One button per tab, in order; a click selects that tab.
pub fn buttons(ui: &mut egui::Ui, current: SettingsTab, out: &mut Vec<Msg>) {
    let items: Vec<rail::Item<SettingsTab>> = SHOWN
        .into_iter()
        .map(|tab| (tab, tab_label(tab), "Shows this group of settings."))
        .collect();
    if let Some(tab) = rail::buttons(ui, &items, current) {
        out.push(Msg::SettingsTab(SettingsTabMsg::Select(tab)));
    }
}
