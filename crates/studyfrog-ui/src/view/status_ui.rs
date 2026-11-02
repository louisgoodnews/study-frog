//! Item status on screen: the menu on the item page, the label on list rows and a stack's roll-up.

use eframe::egui::{self, ComboBox, RichText, Role, WidgetInfo};
use studyfrog_core::status::{STATUSES, Status, status_label};

use crate::theme::palette_of;

/// The text on the status menu: the status the item states, else "Automatic" and what the history makes of it.
pub fn menu_text(stored: Option<Status>, derived: Option<Status>) -> String {
    match (stored, derived) {
        (Some(s), _) => status_label(s).to_owned(),
        (None, Some(d)) => format!("Automatic ({})", status_label(d)),
        (None, None) => "Automatic".to_owned(),
    }
}

/// A drop-down of the statuses and "Automatic"; returns the choice when one was made
/// (`Some(None)` is "Automatic", which leaves the status to the history).
pub fn status_menu(
    ui: &mut egui::Ui,
    stored: Option<Status>,
    derived: Option<Status>,
) -> Option<Option<Status>> {
    let mut picked = None;
    let r = ComboBox::from_id_salt("item-status")
        .selected_text(menu_text(stored, derived))
        .show_ui(ui, |ui| {
            if ui.selectable_label(stored.is_none(), "Automatic").clicked() {
                picked = Some(None);
            }
            for s in STATUSES {
                if ui
                    .selectable_label(stored == Some(s), status_label(s))
                    .clicked()
                {
                    picked = Some(Some(s));
                }
            }
        });
    r.response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Item status"));
    picked
}

/// "3 Learning, 2 In review": the statuses that have items, in menu order.
pub fn summary_text(counts: &[(Status, u32)]) -> Option<String> {
    let parts: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(s, n)| format!("{n} {}", status_label(*s)))
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// A stack's roll-up line under its heading.
pub fn summary(ui: &mut egui::Ui, counts: &[(Status, u32)]) {
    if let Some(text) = summary_text(counts) {
        let weak = palette_of(ui).text_weak;
        ui.label(RichText::new(format!("Status of its items: {text}")).color(weak));
    }
}
