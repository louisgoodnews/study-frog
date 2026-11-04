//! Diary screen view: entries, calendar, editor form, and settings.

pub mod calendar;
pub mod editor;
pub mod editor_ratings;
pub mod home_hint;
pub mod list;
pub mod settings;

use eframe::egui::{self, Role, WidgetInfo};

use crate::diary_msg::DiaryMsg;
use crate::diary_types::DiaryViewMode;
use crate::msg::Msg;
use crate::state::State;

fn diary_msg(out: &mut Vec<Msg>, m: DiaryMsg) {
    out.push(Msg::Diary(m));
}

/// Draw the entire Diary screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.diary.active_entry.is_some() {
        editor::draw(state, ui, out);
        return;
    }

    header_bar(state, ui, out);
    ui.add_space(12.0);

    match state.diary.view_mode {
        DiaryViewMode::List => list::draw(state, ui, out),
        DiaryViewMode::MonthCalendar => calendar::draw(state, ui, out),
    }
}

fn header_bar(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.heading("Diary");
        ui.add_space(16.0);

        let list_sel = state.diary.view_mode == DiaryViewMode::List;
        let list_btn = ui.selectable_label(list_sel, "List");
        list_btn.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Diary list view"));
        if list_btn.clicked() {
            diary_msg(out, DiaryMsg::SetViewMode(DiaryViewMode::List));
        }

        let cal_sel = state.diary.view_mode == DiaryViewMode::MonthCalendar;
        let cal_btn = ui.selectable_label(cal_sel, "Month calendar");
        cal_btn
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Diary month calendar view"));
        if cal_btn.clicked() {
            diary_msg(out, DiaryMsg::SetViewMode(DiaryViewMode::MonthCalendar));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let new_btn = crate::view::kit::primary_button(ui, "+ New entry");
            new_btn
                .widget_info(|| WidgetInfo::labeled(Role::Button, true, "Create new diary entry"));
            if new_btn.clicked() {
                diary_msg(out, DiaryMsg::NewEntry(state.diary.selected_date));
            }
        });
    });
}
