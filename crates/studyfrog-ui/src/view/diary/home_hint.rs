//! Quiet, dismissible Home screen hint when no diary entry exists yet today.

use eframe::egui::{self, Align, Layout, RichText, Role, WidgetInfo};
use studyfrog_core::civil::date_from_days;

use crate::diary_filter::has_entry_for_date;
use crate::diary_msg::DiaryMsg;
use crate::msg::Msg;
use crate::state::{Screen, State};
use crate::theme::palette_of;

/// Draw the quiet "No diary entry yet today" hint on Home if enabled and needed.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if !state.diary.settings.show_home_hint || state.diary.hint_dismissed {
        return;
    }

    let today = date_from_days(state.wall_ms.div_euclid(86_400_000));
    if has_entry_for_date(&state.diary.entries, today) {
        return;
    }

    let p = palette_of(ui);
    crate::view::kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new("No diary entry yet today").color(p.text_weak));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let dismiss_btn = ui.button("Dismiss");
                dismiss_btn
                    .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Dismiss diary hint"));
                if dismiss_btn.clicked() {
                    out.push(Msg::Diary(DiaryMsg::DismissHomeHint));
                }

                let write_btn = crate::view::kit::quiet_button(ui, "Write entry");
                write_btn.widget_info(|| {
                    WidgetInfo::labeled(Role::Button, true, "Write diary entry for today")
                });
                if write_btn.clicked() {
                    out.push(Msg::Diary(DiaryMsg::NewEntry(today)));
                    out.push(Msg::Navigate(Screen::Diary));
                }
            });
        });
    });
}
