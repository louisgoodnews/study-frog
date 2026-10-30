//! A date field with a month picker under it: type the date or pick it.

use eframe::egui::{self, RichText, Role, WidgetInfo};
use studyfrog_core::civil::{Date, date_new, format_date, parse_date};

use crate::planner::grid_of;
use crate::state::State;
use crate::theme::{palette_of, tint};
use crate::view::widgets::text_field;

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn shifted((year, month): (i32, u8), by: i32) -> (i32, u8) {
    let index = year * 12 + i32::from(month) - 1 + by;
    (index.div_euclid(12), (index.rem_euclid(12) + 1) as u8)
}

/// Today's date as far as the screens know it (the home summary, else the calendar's).
pub fn today_of(state: &State) -> Date {
    let fallback = date_new(2026, 1, 1).unwrap_or(Date {
        year: 2026,
        month: 1,
        day: 1,
    });
    match (&state.home, state.planner.loaded) {
        (Some(h), _) => h.today,
        (None, true) => state.planner.today,
        _ => fallback,
    }
}

/// The labelled text field and a "Pick date" button that opens a month grid. Returns the new text
/// when the user typed or picked a date.
pub fn date_field(ui: &mut egui::Ui, label: &str, value: &str, today: Date) -> Option<String> {
    let mut changed = text_field(ui, label, value, "2026-12-31", 1);
    let id = ui.make_persistent_id(("date-picker", label));
    let open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let toggle = if open {
        "Close the date picker"
    } else {
        "Pick date"
    };
    if ui.small_button(toggle).clicked() {
        ui.data_mut(|d| d.insert_temp(id, !open));
    }
    if !open {
        return changed;
    }
    let current = parse_date(changed.as_deref().unwrap_or(value).trim());
    let start = current.unwrap_or(today);
    let month_id = id.with("month");
    let month = ui
        .data(|d| d.get_temp::<(i32, u8)>(month_id))
        .unwrap_or((start.year, start.month));
    if let Some(picked) = month_grid(ui, month, (current, today), month_id) {
        changed = Some(format_date(picked));
        ui.data_mut(|d| d.insert_temp(id, false));
    }
    changed
}

fn month_grid(
    ui: &mut egui::Ui,
    month: (i32, u8),
    (selected, today): (Option<Date>, Date),
    month_id: egui::Id,
) -> Option<Date> {
    let pal = palette_of(ui);
    ui.horizontal(|ui| {
        if ui.small_button("Earlier month").clicked() {
            ui.data_mut(|d| d.insert_temp(month_id, shifted(month, -1)));
        }
        ui.strong(format!("{} {}", MONTHS[usize::from(month.1) - 1], month.0));
        if ui.small_button("Later month").clicked() {
            ui.data_mut(|d| d.insert_temp(month_id, shifted(month, 1)));
        }
    });
    let mut picked = None;
    egui::Grid::new(month_id.with("grid"))
        .num_columns(7)
        .show(ui, |ui| {
            for name in DAY_NAMES {
                ui.weak(name);
            }
            ui.end_row();
            for (i, day) in grid_of(month).into_iter().enumerate() {
                let mut text = RichText::new(day.day.to_string());
                if (day.year, day.month) != month {
                    text = text.color(pal.text_weak);
                }
                let mut button = egui::Button::new(text).small();
                if Some(day) == selected {
                    button = button.fill(tint(pal.accent, 90));
                } else if day == today {
                    button = button.stroke(egui::Stroke::new(1.0, pal.accent));
                }
                let response = ui.add(button);
                response.widget_info(|| WidgetInfo::labeled(Role::Button, true, format_date(day)));
                if response.clicked() {
                    picked = Some(day);
                }
                if i % 7 == 6 {
                    ui.end_row();
                }
            }
        });
    picked
}
