//! Month calendar view of diary entries.

use eframe::egui::{self, RichText, Role, WidgetInfo, vec2};
use studyfrog_core::civil::{Date, Weekday, format_date, month_grid};

use crate::diary_filter::entries_for_date;
use crate::diary_msg::DiaryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;

const MONTH_NAMES: [&str; 12] = [
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
const WEEKDAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn diary_msg(out: &mut Vec<Msg>, m: DiaryMsg) {
    out.push(Msg::Diary(m));
}

/// Draw month calendar grid and the day entry pane.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    let selected = state.diary.selected_date;

    ui.horizontal(|ui| {
        let prev = ui.button("<");
        prev.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Previous month"));
        if prev.clicked() {
            diary_msg(out, DiaryMsg::StepMonth(-1));
        }

        let m_idx = (selected.month.clamp(1, 12) - 1) as usize;
        let month_label = format!("{} {}", MONTH_NAMES[m_idx], selected.year);
        ui.heading(month_label);

        let next = ui.button(">");
        next.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Next month"));
        if next.clicked() {
            diary_msg(out, DiaryMsg::StepMonth(1));
        }
    });

    ui.add_space(8.0);

    // Weekday headers
    ui.columns(7, |cols| {
        for (i, col) in cols.iter_mut().enumerate() {
            col.label(RichText::new(WEEKDAY_NAMES[i]).strong().color(p.text_weak));
        }
    });

    ui.add_space(4.0);

    // 42-day calendar grid
    let grid = month_grid(selected.year, selected.month, Weekday::Mon);
    for week in 0..6 {
        ui.columns(7, |cols| {
            for day_idx in 0..7 {
                let cell_date = grid[week * 7 + day_idx];
                let is_current_month = cell_date.month == selected.month;
                let is_selected = cell_date == selected;
                let day_entries = entries_for_date(&state.diary.entries, cell_date);
                let col = &mut cols[day_idx];

                let mut btn_text = format!("{}", cell_date.day);
                if !day_entries.is_empty() {
                    btn_text.push_str(&format!(" ({})", day_entries.len()));
                }

                let rich = if is_selected {
                    RichText::new(btn_text).strong().color(p.background)
                } else if is_current_month {
                    RichText::new(btn_text).color(p.text)
                } else {
                    RichText::new(btn_text).color(p.text_weak)
                };

                let a11y_name = format!(
                    "Select date {} with {} entries",
                    format_date(cell_date),
                    day_entries.len()
                );

                let btn = if is_selected {
                    egui::Button::new(rich).fill(p.accent)
                } else if !day_entries.is_empty() {
                    egui::Button::new(rich).fill(p.surface)
                } else {
                    egui::Button::new(rich)
                }
                .min_size(vec2(col.available_width(), 28.0));

                let resp = col.add(btn);
                resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, &a11y_name));
                if resp.clicked() {
                    diary_msg(out, DiaryMsg::SelectDate(cell_date));
                }
            }
        });
        ui.add_space(2.0);
    }

    ui.add_space(16.0);
    day_entries_pane(state, selected, ui, out);
}

fn day_entries_pane(state: &State, date: Date, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    let day_entries = entries_for_date(&state.diary.entries, date);

    ui.horizontal(|ui| {
        ui.heading(format!("Entries for {}", format_date(date)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let new_btn = crate::view::kit::primary_button(ui, "+ Write entry for this day");
            new_btn.widget_info(|| {
                WidgetInfo::labeled(
                    Role::Button,
                    true,
                    format!("Write entry for {}", format_date(date)),
                )
            });
            if new_btn.clicked() {
                diary_msg(out, DiaryMsg::NewEntry(date));
            }
        });
    });

    ui.add_space(8.0);

    if day_entries.is_empty() {
        ui.weak(format!(
            "No diary entries recorded for {}.",
            format_date(date)
        ));
        return;
    }

    for entry in day_entries {
        crate::view::kit::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(&entry.title);
                if let Some(mins) = entry.study_minutes {
                    crate::view::kit::pill(ui, &format!("{mins} min"), p.accent);
                }
                for s in &entry.stacks {
                    crate::view::kit::pill(ui, s, p.text_weak);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let open_btn = ui.button("Open in Editor");
                    open_btn.widget_info(|| {
                        WidgetInfo::labeled(
                            Role::Button,
                            true,
                            format!("Open {} in Editor", entry.title),
                        )
                    });
                    if open_btn.clicked() {
                        out.push(Msg::OpenItem(entry.uid));
                    }

                    let edit_btn = ui.button("Edit");
                    edit_btn.widget_info(|| {
                        WidgetInfo::labeled(Role::Button, true, format!("Edit {}", entry.title))
                    });
                    if edit_btn.clicked() {
                        diary_msg(out, DiaryMsg::EditEntry(entry.uid));
                    }
                });
            });

            if !entry.ratings.is_empty() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for (k, v) in &entry.ratings {
                        let text = match v {
                            studyfrog_core::diary::RatingValue::Int(n) => format!("{k}: {n}/5"),
                            studyfrog_core::diary::RatingValue::Text(t) => format!("{k}: {t}"),
                        };
                        crate::view::kit::pill(ui, &text, p.alt);
                    }
                });
            }

            if !entry.content.trim().is_empty() {
                ui.add_space(4.0);
                let preview: String = entry.content.lines().take(3).collect::<Vec<_>>().join(" ");
                ui.label(RichText::new(preview).italics().color(p.text_weak));
            }
        });
        ui.add_space(6.0);
    }
}
